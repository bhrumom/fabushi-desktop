import Foundation
import AVFoundation
import CoreGraphics
import CoreVideo
import ImageIO

private enum RecorderError: Error, CustomStringConvertible {
    case usage(String)
    case noFrames
    case imageLoad(String)
    case pixelBuffer
    case context
    case writer(String)
    case append(Int)
    case invalidVideo(String)

    var description: String {
        switch self {
        case .usage(let message): return message
        case .noFrames: return "no capture frames were found"
        case .imageLoad(let path): return "unable to decode image: \(path)"
        case .pixelBuffer: return "unable to allocate video pixel buffer"
        case .context: return "unable to create CoreGraphics pixel context"
        case .writer(let message): return "AVAssetWriter failed: \(message)"
        case .append(let index): return "unable to append frame \(index)"
        case .invalidVideo(let message): return "video validation failed: \(message)"
        }
    }
}

private func frameURLs(_ directory: URL) throws -> [URL] {
    let extensions = Set(["jpg", "jpeg", "png"])
    return try FileManager.default.contentsOfDirectory(
        at: directory,
        includingPropertiesForKeys: nil,
        options: [.skipsHiddenFiles]
    ).filter { extensions.contains($0.pathExtension.lowercased()) }
     .sorted { $0.lastPathComponent < $1.lastPathComponent }
}

private func loadImage(_ url: URL) throws -> CGImage {
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
        throw RecorderError.imageLoad(url.path)
    }
    return image
}

private func even(_ value: Int) -> Int {
    max(2, value - (value % 2))
}

private func outputSize(for image: CGImage) -> (Int, Int) {
    let maxWidth = 1280.0
    let scale = min(1.0, maxWidth / Double(image.width))
    return (
        even(Int((Double(image.width) * scale).rounded(.down))),
        even(Int((Double(image.height) * scale).rounded(.down)))
    )
}

private func pixelBuffer(from image: CGImage, width: Int, height: Int) throws -> CVPixelBuffer {
    let attributes: [String: Any] = [
        kCVPixelBufferCGImageCompatibilityKey as String: true,
        kCVPixelBufferCGBitmapContextCompatibilityKey as String: true,
        kCVPixelBufferWidthKey as String: width,
        kCVPixelBufferHeightKey as String: height,
        kCVPixelBufferPixelFormatTypeKey as String: Int(kCVPixelFormatType_32ARGB),
    ]
    var maybeBuffer: CVPixelBuffer?
    guard CVPixelBufferCreate(
        kCFAllocatorDefault,
        width,
        height,
        kCVPixelFormatType_32ARGB,
        attributes as CFDictionary,
        &maybeBuffer
    ) == kCVReturnSuccess, let buffer = maybeBuffer else {
        throw RecorderError.pixelBuffer
    }

    CVPixelBufferLockBaseAddress(buffer, [])
    defer { CVPixelBufferUnlockBaseAddress(buffer, []) }
    guard let base = CVPixelBufferGetBaseAddress(buffer) else { throw RecorderError.pixelBuffer }
    guard let context = CGContext(
        data: base,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: CVPixelBufferGetBytesPerRow(buffer),
        space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue
    ) else {
        throw RecorderError.context
    }
    context.interpolationQuality = .medium
    context.setFillColor(CGColor(gray: 0, alpha: 1))
    context.fill(CGRect(x: 0, y: 0, width: width, height: height))
    context.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
    return buffer
}

private func encode(framesDirectory: URL, output: URL, durationSeconds: Double) throws {
    let frames = try frameURLs(framesDirectory)
    guard !frames.isEmpty else { throw RecorderError.noFrames }
    let first = try loadImage(frames[0])
    let (width, height) = outputSize(for: first)
    try? FileManager.default.removeItem(at: output)

    let writer = try AVAssetWriter(outputURL: output, fileType: .mov)
    let settings: [String: Any] = [
        AVVideoCodecKey: AVVideoCodecType.h264,
        AVVideoWidthKey: width,
        AVVideoHeightKey: height,
        AVVideoCompressionPropertiesKey: [
            AVVideoAverageBitRateKey: 2_500_000,
            AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
        ],
    ]
    let input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
    input.expectsMediaDataInRealTime = false
    let adaptor = AVAssetWriterInputPixelBufferAdaptor(
        assetWriterInput: input,
        sourcePixelBufferAttributes: [
            kCVPixelBufferPixelFormatTypeKey as String: Int(kCVPixelFormatType_32ARGB),
            kCVPixelBufferWidthKey as String: width,
            kCVPixelBufferHeightKey as String: height,
        ]
    )
    guard writer.canAdd(input) else { throw RecorderError.writer("video input is unsupported") }
    writer.add(input)
    guard writer.startWriting() else {
        throw RecorderError.writer(writer.error?.localizedDescription ?? "startWriting returned false")
    }
    writer.startSession(atSourceTime: .zero)

    let totalDuration = max(0.25, durationSeconds)
    let denominator = max(1, frames.count - 1)
    for (index, url) in frames.enumerated() {
        while !input.isReadyForMoreMediaData {
            if writer.status == .failed {
                throw RecorderError.writer(writer.error?.localizedDescription ?? "writer failed while waiting for input")
            }
            Thread.sleep(forTimeInterval: 0.01)
        }
        let image = try loadImage(url)
        let buffer = try pixelBuffer(from: image, width: width, height: height)
        let seconds = Double(index) * totalDuration / Double(denominator)
        let timestamp = CMTime(seconds: seconds, preferredTimescale: 600)
        guard adaptor.append(buffer, withPresentationTime: timestamp) else {
            throw RecorderError.append(index)
        }
    }
    input.markAsFinished()
    let done = DispatchSemaphore(value: 0)
    writer.finishWriting { done.signal() }
    done.wait()
    guard writer.status == .completed else {
        throw RecorderError.writer(writer.error?.localizedDescription ?? "finishWriting did not complete")
    }
}

private func validate(_ url: URL) throws {
    let asset = AVURLAsset(url: url)
    let tracks = asset.tracks(withMediaType: .video)
    guard let track = tracks.first else { throw RecorderError.invalidVideo("no video track") }
    let duration = CMTimeGetSeconds(asset.duration)
    guard duration.isFinite && duration > 0 else {
        throw RecorderError.invalidVideo("duration is not positive")
    }
    let reader = try AVAssetReader(asset: asset)
    let output = AVAssetReaderTrackOutput(track: track, outputSettings: nil)
    guard reader.canAdd(output) else { throw RecorderError.invalidVideo("reader cannot add video output") }
    reader.add(output)
    guard reader.startReading() else {
        throw RecorderError.invalidVideo(reader.error?.localizedDescription ?? "reader did not start")
    }
    guard output.copyNextSampleBuffer() != nil else {
        throw RecorderError.invalidVideo("first video sample could not be decoded")
    }
    let formattedDuration = String(format: "%.3f", duration)
    print("playable=true duration_seconds=\(formattedDuration) video_tracks=\(tracks.count) first_sample=decoded")
}

private func run() throws {
    let args = Array(CommandLine.arguments.dropFirst())
    guard let command = args.first else {
        throw RecorderError.usage("usage: fcm-010-13-macos-session-video.swift encode <frames-dir> <output.mov> <duration-seconds> | validate <video.mov>")
    }
    switch command {
    case "encode":
        guard args.count == 4, let duration = Double(args[3]), duration > 0 else {
            throw RecorderError.usage("encode requires <frames-dir> <output.mov> <duration-seconds>")
        }
        try encode(
            framesDirectory: URL(fileURLWithPath: args[1]),
            output: URL(fileURLWithPath: args[2]),
            durationSeconds: duration
        )
        try validate(URL(fileURLWithPath: args[2]))
    case "validate":
        guard args.count == 2 else { throw RecorderError.usage("validate requires <video.mov>") }
        try validate(URL(fileURLWithPath: args[1]))
    default:
        throw RecorderError.usage("unknown command: \(command)")
    }
}

do {
    try run()
} catch {
    FileHandle.standardError.write(Data("\(error)\n".utf8))
    exit(1)
}

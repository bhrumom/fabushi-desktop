import Foundation
import CoreGraphics
import AVFoundation
import Darwin

func intArg(_ index: Int, _ label: String) -> Int {
    guard CommandLine.arguments.count > index, let value = Int(CommandLine.arguments[index]) else {
        fputs("invalid \(label)\n", stderr); exit(2)
    }
    return value
}
func point(_ x: Int, _ y: Int) -> CGPoint { CGPoint(x: x, y: y) }
func postMouse(_ type: CGEventType, _ p: CGPoint, _ button: CGMouseButton = .left) {
    guard let event = CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: p, mouseButton: button) else { exit(3) }
    event.post(tap: .cghidEventTap)
}

final class MovieRecordingDelegate: NSObject, AVCaptureFileOutputRecordingDelegate {
    let finished = DispatchSemaphore(value: 0)
    var recordingError: Error?

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        recordingError = error
        finished.signal()
    }
}

func recordingActuallyFailed(_ error: Error?) -> Bool {
    guard let error else { return false }
    let nsError = error as NSError
    if let finished = nsError.userInfo[AVErrorRecordingSuccessfullyFinishedKey] as? Bool, finished {
        return false
    }
    return true
}

func recordScreen(_ outputPath: String, _ maxSeconds: Int) {
    let outputURL = URL(fileURLWithPath: outputPath)
    try? FileManager.default.removeItem(at: outputURL)

    guard let input = AVCaptureScreenInput(displayID: CGMainDisplayID()) else {
        fputs("unable to create screen capture input\n", stderr); exit(4)
    }
    input.minFrameDuration = CMTime(value: 1, timescale: 15)
    input.capturesCursor = true
    input.capturesMouseClicks = true

    let session = AVCaptureSession()
    guard session.canAddInput(input) else {
        fputs("unable to attach screen capture input\n", stderr); exit(4)
    }
    session.addInput(input)

    let output = AVCaptureMovieFileOutput()
    guard session.canAddOutput(output) else {
        fputs("unable to attach movie output\n", stderr); exit(4)
    }
    session.addOutput(output)

    let delegate = MovieRecordingDelegate()
    let controlQueue = DispatchQueue(label: "com.ombhrum.fabushi.screen-recording")
    var stopRequested = false

    signal(SIGINT, SIG_IGN)
    signal(SIGTERM, SIG_IGN)
    let sigInt = DispatchSource.makeSignalSource(signal: SIGINT, queue: controlQueue)
    let sigTerm = DispatchSource.makeSignalSource(signal: SIGTERM, queue: controlQueue)
    let timer = DispatchSource.makeTimerSource(queue: controlQueue)

    let requestStop = {
        if stopRequested { return }
        stopRequested = true
        if output.isRecording {
            output.stopRecording()
        } else {
            delegate.finished.signal()
        }
    }
    sigInt.setEventHandler(handler: requestStop)
    sigTerm.setEventHandler(handler: requestStop)
    timer.setEventHandler(handler: requestStop)
    sigInt.resume()
    sigTerm.resume()
    timer.schedule(deadline: .now() + .seconds(max(1, maxSeconds)))

    session.startRunning()
    output.startRecording(to: outputURL, recordingDelegate: delegate)
    delegate.finished.wait()

    timer.cancel()
    sigInt.cancel()
    sigTerm.cancel()
    session.stopRunning()

    if recordingActuallyFailed(delegate.recordingError) {
        fputs("screen recording failed: \(delegate.recordingError!)\n", stderr)
        exit(5)
    }
    guard let attrs = try? FileManager.default.attributesOfItem(atPath: outputPath),
          let size = attrs[.size] as? NSNumber,
          size.int64Value > 0 else {
        fputs("screen recording produced no video; grant Screen Recording permission to Fabushi\n", stderr)
        exit(6)
    }
}

guard CommandLine.arguments.count >= 2 else { fputs("command required\n", stderr); exit(2) }
switch CommandLine.arguments[1] {
case "move":
    let x=intArg(2,"x"), y=intArg(3,"y"); postMouse(.mouseMoved, point(x,y))
case "drag":
    let x1=intArg(2,"x1"), y1=intArg(3,"y1"), x2=intArg(4,"x2"), y2=intArg(5,"y2")
    let duration=max(40,intArg(6,"durationMs")), steps=max(2,min(120,duration/16))
    postMouse(.mouseMoved,point(x1,y1)); postMouse(.leftMouseDown,point(x1,y1))
    for step in 1...steps {
        let t=Double(step)/Double(steps)
        let p=CGPoint(x:Double(x1)+(Double(x2-x1)*t),y:Double(y1)+(Double(y2-y1)*t))
        postMouse(.leftMouseDragged,p); usleep(useconds_t(max(1000,duration*1000/steps)))
    }
    postMouse(.leftMouseUp,point(x2,y2))
case "scroll":
    let dx=intArg(2,"dx"), dy=intArg(3,"dy")
    guard let event=CGEvent(scrollWheelEvent2Source:nil,units:.pixel,wheelCount:2,wheel1:Int32(dy),wheel2:Int32(dx),wheel3:0) else { exit(3) }
    event.post(tap:.cghidEventTap)
case "record":
    guard CommandLine.arguments.count >= 3 else { fputs("record output path required\n", stderr); exit(2) }
    let maxSeconds = CommandLine.arguments.count >= 4 ? max(1, intArg(3, "maxSeconds")) : 600
    recordScreen(CommandLine.arguments[2], maxSeconds)
default:
    fputs("unknown command\n", stderr); exit(2)
}

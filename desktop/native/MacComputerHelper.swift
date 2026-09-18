import Foundation
import CoreGraphics

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
default:
    fputs("unknown command\n", stderr); exit(2)
}

import {Point, Value, FrontendMessage, MyBytes} from './pkg/mywasm.d';


let x : Point  =  { X:1, Y: 2, z: 3 };

let y : Value<number> = {value:32}

let s = "s"
let z : FrontendMessage = { tag: "ButtonState" , fields: { selected: ["a"] , time: 32. , other: s } }

let q : any = JSON.parse("{ x:1, Y: 2, z: 3, q:5 }");

let r = q as Point;
r.X;

function isPoint(obj: any): obj is Point {
    return obj.X && obj.Y && obj.z && typeof obj.X == "number"
}

if (isPoint(q)) {
    // let r = q as Point;
    q.z
}

function isValue<T>(x: any): x is Value<T> {
    return x.value ! == undefined && typeof x.value === typeof T
}

if (isValue<number>(y)) {

}

// cryptoの挙動確認
const plain =
  "ahAu4pp599+03jvJ5s0o+g==" + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

//   https://note.kiriukun.com/entry/20190822-generating-a-sha256-hash-in-nodejs
const key = require("crypto").createHash("sha1").update(plain).digest("hex");

console.log(plain);
console.log(key);

const messages = document.getElementById("messages");

const address = window.location.href.split("//").at(1).split(":").at(0);         
const port = Number(window.location.href.split("//").at(1).split(":").at(1).split("/").at(0)) + 1;
const socketAddress = `${address}:${port}`;
console.log(socketAddress);
let socket = new WebSocket(`ws://${socketAddress}`);
socket.onopen = () => {
	socket.send("Hello from client (but waiting patiently)");
}
socket.onmessage = (message) => {
	let data = document.createElement("li");
	data.innerHTML = message.data;
	messages.appendChild(data);
}
const body = document.getElementsByTagName("body")[0];
/*for (let i = 0; i < body.children.length; i++) {
	body.children[i].style.opacity = 1;
}*/
body.style.opacity = 1;

const messages = document.getElementById("messages");

const address = window.location.href.split("//").at(1).split(":").at(0);         
const port = Number(window.location.href.split("//").at(1).split(":").at(1).split("/").at(0)) + 1;
const socketAddress = `${address}:${port}`;
console.log(socketAddress);
let socket = new WebSocket(`ws://${socketAddress}`);
socket.onopen = async () => {
	socket.send("Hello from client (but waiting patiently)");
	let events = await fetch("events");
	let eventsText = await events.text();
	console.log(eventsText);
}
socket.onmessage = (message) => {
	let data = document.createElement("li");
	data.innerHTML = message.data;
	messages.appendChild(data);
	let messageSplit = message.data.split(":");
	let command = messageSplit[0];
	let content = messageSplit[1];
	switch (command) {
		case "events":
			console.log("New event");
			console.log(content);
			break;
		case "cmd":
			console.log("New command");
			console.log(content);
			break;
		default:
			console.log("Bogus amogus command sent from the server");
			console.log(message.data);
			break;
	}
}
const body = document.getElementsByTagName("body")[0];
/*for (let i = 0; i < body.children.length; i++) {
	body.children[i].style.opacity = 1;
}*/
body.style.opacity = 1;

/*async function startStream() {
	let response = await fetch("/startstream", {Headers: {
		Content-Type: "test/event-stream",
		Method: "POST"
	}});	
	if (response.ok) { 
  		let text = await response.text();
  		console.log(text);
		sortResponses(text);
	}
}
function sortResponses(text) {
	console.log("sorting responses: " + text);
}
let i = 0;
awaitNextResponse();*/
/*async function startSocket() {
	let socket = new WebSocket("ws://192.168.1.170:6970/stream");
	console.log(socket);
	socket.onmessage = (event) => console.log(event);
}
startSocket();*/
let socket = localStorage.getItem("webSocket");
socket.onopen = () => {
	socket.send("Hello from client (but waiting patiently)");
}
const body = document.getElementsByTagName("body")[0];
for (let i = 0; i < body.children.length; i++) {
	body.children[i].style.opacity = 1;
}

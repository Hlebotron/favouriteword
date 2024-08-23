function sendAlert() {
	let textbox = document.getElementById('alert');
	fetch('message', {
		method: "POST",
		body: textbox.value
	});
	textbox.value = "";
}
const ipSplit = window.location.href.split("//").at(1).split(":");
const address = ipSplit.at(0);
const port = Number(ipSplit.at(1).slice(0, -1)) + 1;
const socketAddress = `${address}:${port}`;
let socket = new WebSocket(`ws://${socketAddress}`);
socket.onopen = async () => {
	socket.send("pogger");
	let events = await fetch("getAnswerData");
	let eventsText = await events.text();
	console.log("eventstext" + eventsText);
	let eventLines = eventsText.split("\n");
	for (i = 0; i < eventLines.length; i++) {
		let eventLine = eventLines[i];
		if (eventLine == "") { 
			console.log("empty");
			continue;
		}
		console.log("eventLine: " + eventLine);
	}
}
socket.onmessage = (message) => {
	let messageSplit = message.data.split(":");
	let command = messageSplit.at(0);
	let content = messageSplit.at(1);
	switch (command) {
		case "alert":
			alert(content);
	}	
}

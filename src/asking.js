const messages = document.getElementById("messages");
const answer = document.getElementById("clientAnswer");
const asking = document.getElementById("asking");
const wordElement = document.getElementById("word");
const waiting = document.getElementById("waiting");

const address = window.location.href.split("//").at(1).split(":").at(0);         
const port = Number(window.location.href.split("//").at(1).split(":").at(1).split("/").at(0)) + 1;
const socketAddress = `${address}:${port}`;
console.log(socketAddress);
let socket = new WebSocket(`ws://${socketAddress}`);
let startAsking = false;
let stopAsking = false;
let isWaiting = localStorage.getItem("isWaiting");
if (isWaiting == null) {
	localStorage.setItem("isWaiting", "true");
	isWaiting = "true";
}
let word = "";
let isSent = localStorage.getItem("isSent");
if (isSent == null) {
	localStorage.setItem("isSent", "false");
	isSent = "false";
}
console.log("isSent: " + isSent);
let clientAnswer = "";
socket.onopen = async () => {
	socket.send("Hello from client (but waiting patiently)");
	let events = await fetch("events");
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
		let splitEvent = eventLine.split(":");
		let command = splitEvent[0];
		let content = splitEvent[1];
		switch (command) {
			case "word":
				console.log("New word: " + content);
				word = content;
				startAsking = true;
				break;
			case "cmd":
				console.log("New command: " + content);
				switch (content) {
					case "stopAsking":
						stopAsking = true;
						break;
					case "reset":
						localStorage.clear();
						break;
				}
				break;
			case "msg":
				console.log("New message: " + content);
				alert(content);
				break;
		}
	}
	//console.log("startAsking: " + startAsking);
	//console.log("stopAsking: " + stopAsking);
	if (isWaiting == "false") {
		if (isWaiting == "false" && !stopAsking) {
			asking.style.opacity = 1;
			wordElement.innerHTML = "The word is: " + word;
		} else {
			alert("Thank you for participating");
			document.getElementById("end").style.opacity = 1;
		}
	} else {
		waiting.style.opacity = 1;
		if (startAsking) {
			document.getElementById("answerDisplay").innerHTML = "Your answer is: " + clientAnswer;
		}
	}
}
socket.onmessage = (message) => {
	let data = document.createElement("li");
	data.innerHTML = message.data;
	messages.appendChild(data);
	let messageSplit = message.data.split(":");
	let command = messageSplit[0];
	let content = messageSplit[1];
	switch (command) {
		case "word":
			console.log("New word: " + content);
			word = content;
			document.getElementById("word").innerHTML = "The word is: " + word;
			startAsking = true;
			waiting.style.opacity = 0;
			setTimeout(() => document.getElementById("asking").style.opacity = 1, 250);
			isWaiting = "false";
			localStorage.setItem("isWaiting", "false");
			isSent = "false";
			break;
		case "cmd":
			console.log("New command: " + content);
			switch (content) {
				case "stopAsking":
					asking.opacity = 0;
					setTimeout(() => document.getElementById("end").style.opacity = 1, 250);
					stopAsking = true;
					break;
				case "reset":
					localStorage.clear();
					break;
			}
			break;
		case "msg":
			console.log("New message: " + content);
			alert(content);
			break;
		case "response":
			console.log("Response from server");
			break;
		default:
			console.log("Bogus amogus command sent from the server");
			console.log(message.data);
			break;
	}
}
function submitAnswer() {
	let answer = document.getElementById('clientAnswer').value;
	switch (isSent) {
		case "false":
			fetch("addAnswerData", {body: answer, method: "POST"});
			isSent = "true";
			localStorage.setItem("isSent", "true");
			console.log("sent");
			break;
		case "true":
			console.log("isSent");
			break;
		default:
			localStorage.setItem("isSent", "false");
			isSent = "false";
			break;

	}
	isWaiting = "true";
	localStorage.setItem("isWaiting", "true");
	asking.style.opacity = 0;
	answerDisplay.innerHTML = "Your answer is: " + answer;
	setTimeout(() => waiting.style.opacity = 1, 250);
}

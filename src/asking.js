const messages = document.getElementById("messages");
const answer = document.getElementById("clientAnswer");
const asking = document.getElementById("asking");
const wordElement = document.getElementById("word");
const waiting = document.getElementById("waiting");
const realAnswer = document.getElementById("realAnswer");

const address = window.location.href.split("//").at(1).split(":").at(0);         
const port = Number(window.location.href.split("//").at(1).split(":").at(1).split("/").at(0)) + 1;
const socketAddress = `${address}:${port}`;
console.log(socketAddress);
let socket = new WebSocket(`ws://${socketAddress}`);
let startAsking = false;
let isWaiting = localStorage.getItem("isWaiting");
if (isWaiting == null) {
	localStorage.setItem("isWaiting", "true");
	isWaiting = "true";
}
let word = "";
let name = "";
let isSent = localStorage.getItem("isSent");
if (isSent == null) {
	localStorage.setItem("isSent", "false");
	isSent = "false";
}
console.log("isSent: " + isSent);
let clientAnswer = "";
let clientAnswerLS = localStorage.getItem("clientAnswer");
switch (clientAnswerLS) {
	case null: case undefined: case "":
		break;
	default:
		clientAnswer = clientAnswerLS;
		break;

}
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
		let splitEvent = eventLine.split(":");
		let command = splitEvent[0];
		let content = splitEvent[1];
		switch (command) {
			case "word":
				console.log("New word: " + content);
				let splitContent = content.split("&");
				name = splitContent.at(0);
				word = splitContent.at(1);
				startAsking = true;
				realAnswer.innerHTML = "";
				break;
			case "cmd":
				console.log("New command: " + content);
				switch (content) {
					case "reset":
						localStorage.clear();
						break;
					case "reveal":
						realAnswer.innerHTML = name;
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
	if (isWaiting == "false" && clientAnswer != "") {
		if (isWaiting == "false") {
			asking.style.opacity = 1;
			wordElement.innerHTML = "The word is: " + word;
		} else {
			alert("Thank you for participating");
			document.getElementById("end").style.opacity = 1;
		}
	} else {
		waiting.style.opacity = 1;
		if (startAsking && clientAnswer != "") {
			document.getElementById("answerDisplay").innerHTML = "Your answer is: " + clientAnswer;
		}
	}
}
socket.onmessage = (message) => {
	let data = document.createElement("li");
	data.innerHTML = message.data;
	let messageSplit = message.data.split(":");
	let command = messageSplit[0];
	let content = messageSplit[1];
	switch (command) {
		case "word":
			console.log("New word: " + content);
			let splitContent = content.split("&");
			name = splitContent.at(0);
			word = splitContent.at(1);
			document.getElementById("word").innerHTML = "The word is: " + word;
			startAsking = true;
			waiting.style.opacity = 0;
			setTimeout(() => document.getElementById("asking").style.opacity = 1, 250);
			isWaiting = "false";
			localStorage.setItem("isWaiting", "false");
			isSent = "false";
			realAnswer.innerHTML = "";
			break;
		case "cmd":
			console.log("New command: " + content);
			switch (content) {
				case "reset":
					localStorage.clear();
					break;
				case "reveal":
					realAnswer.innerHTML = name;
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
			localStorage.setItem("clientAnswer", answer);
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

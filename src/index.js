const nameElement = document.getElementById("name");
const wordElement = document.getElementById("desc");
const div = document.getElementsByTagName("body")[0];

const address = window.location.href.split("//").at(1).split(":").at(0);
const port = Number(window.location.href.split("//").at(1).split(":").at(1).split("/").at(0)) + 1;
const socketAddress = `${address}:${port}`;

let socket = new WebSocket(`ws://${socketAddress}`);
socket.onopen = () => {
	socket.send("Hello from client");
} 
socket.onmessage = (message) => {
	let splitMessage = message.data.split(":");	
	let command = splitMessage[0];
	let content = splitMessage[1];
	if (command == "msg") {
		alert(content);
	}
}

let isSent = localStorage.getItem("isSent");
if (isSent == null) {
	localStorage.setItem("isSent", "false");
	isSent = "false";
	console.log("Local storage is empty");
}
console.log(isSent);

function sanitize() {
	let name = nameElement.value;
	let word = wordElement.value;

	let nameSplit = name.split('');
	let nameSanitized = "";
	for (i = 0; i < nameSplit.length;) {
		if (nameSplit[i] == '&' || nameSplit[i] == '=') {
			nameSplit.splice(i, 1);
		} else {
			nameSanitized += nameSplit[i];
			i++;
		}
	}

	let wordSplit = word.split('');
	let wordSanitized = "";
	for (i = 0; i < wordSplit.length;) {
		if (wordSplit[i] == '&' || wordSplit[i] == '=') {
			wordSplit.splice(i, 1);
		} else {
			wordSanitized += wordSplit[i];
			i++;
		}
	}

	nameElement.value = nameSanitized;
	wordElement.value = wordSanitized;
}
function refreshPage() {
	if (nameElement.value != "" && wordElement.value != "") {
		div.style.opacity = 0;
		setTimeout(() => {
			window.open("/asking", "_self");
		}, 500);
	} else {
		alert("One or more fields are empty");
	}
}
function addData() {
	//alert(removeAmpersand("name"));
	sanitize();
	filterNewLines();
	console.log(nameElement.value + " " + wordElement.value);
	if (isSent == "false" && nameElement.value != "" && wordElement.value != "") {
		let data = nameElement.value + "&" + wordElement.value;
		console.log(data);
		fetch("/addData", { 
			method: "POST", 
			body: data
		});
		localStorage.setItem("isSent", "true");
	} else {
		console.log("Not sent");
		alert("Nem lett elküldve");
	}
}
/*function removeAmpersand(elementId) {
	let elementValue = document.getElementById(elementId).value;
	let split = elementValue.split('');
	let sanitized = "";
	console.log(sanitized);
	for (i = 0; i < split.length;) {
		if (split[i] == '&') {
			split.splice(i, 1);
		} else {
			sanitized += split[i];
			i++;
		}
	}
	console.debug(sanitized);
	return sanitized;
}*/
div.style.opacity = 1;
function filterNewLines() {
	let desc = document.getElementById('desc');
	let unfiltered = desc.value;
	let filtered = unfiltered.split("\n").join("\\n");
	desc.value = filtered;
}


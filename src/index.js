const nameElement = document.getElementById("name");
const wordElement = document.getElementById("word");
const body = document.getElementsByTagName("body")[0];

const address = window.location.href.split("//").at(1).split(":").at(0);
const port = Number(window.location.href.split("//").at(1).split(":").at(1).split("/").at(0)) + 1;
const socketAddress = `${address}:${port}`;
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
	body.style.opacity = 0;
	setTimeout(() => {
		window.open("/waiting", "_self");
	}, 500);
}
/*function substituteContent() {
	console.log(body.children[0]);
	for (let i = 0; i < body.children.length;) {
		body.removeChild(body.children[i]);
	}
	console.log(body.children[0]);
	let title = document.createElement("h1");
		title.innerHTML = "Please wait...";
		body.appendChild(title);
	//let html = fetch("/waiting.html", method: "GET");
}*/
function addData() {
	sanitize();
	fetch("/adddata", { 
		method: "POST", 
		body: nameElement.value + "&" + wordElement.value 
	});
}
//let socket = localStorage.getItem("webSocket");
//console.log(socket);
let socket = new WebSocket(`ws://${socketAddress}`);
/*if (localStorage.length == 0) {
	socket = new WebSocket(`ws://${socketAddress}`);
}*/
//console.log(Object.keys(socket));
//localStorage.setItem("webSocket", socket);
//localStorage.clear();
socket.onopen = () => {
	socket.send("Hello from Client");
	setTimeout(() => {socket.send("delayed")}, 3000);
}
socket.onmessage = (message) => {
	console.log(message.data);
	document.getElementById("serverEvent").innerHTML = message.data;
}
window.onbeforeunload = () => {
	socket.close();
}
setInterval(sanitize, 1000);

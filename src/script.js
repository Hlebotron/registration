const nameElement = document.getElementById("name");
const inputDiv = document.getElementById("input");
const inputTitle = document.getElementById("inputTitle");
const validationDiv = document.getElementById("validation");
const inputButton = document.getElementById("inputButton");

async function validate(name) {
	if (name != "" && name != null) {
		localStorage.setItem("name", name);
		inputDiv.style.opacity = 0;
		validationDiv.style.opacity = 1;
		inputDiv.style.zIndex = 0;
		validationDiv.style.zIndex = 1;
		let response = await fetch("validate", {
			method: "POST",
			body: nameElement.value
		});
		let text = await response.text();
	} else {
		alert("Please input your name again");
	}
}
async function switchName() {
	validationDiv.style.opacity = 0;
	validationDiv.style.zIndex = 0;
	inputDiv.style.opacity = 1;
	inputDiv.style.zIndex = 1;
	inputButton.onclick = "confirmSwitch()";
}
async function confirmSwitch() {
	console.log("confirming");
	let confirmation = await validate(nameElement.value);	
	if (confirmation == "pass") {
		fetch("deleteName", {
			body: lsName
		});
	}
}
/*function hide(id) {
	let div = document.getElementById(id);
	let inner = div.innerHTML;
	div.innerHTML = "";
	return inner;
}
function show(id, inner) {
	console.log(inner);
	let div = document.getElementById(id);
	div.innerHTML = inner;
}*/

let lsName = localStorage.getItem("name"); 
if (lsName != null && lsName != "") {
	validationDiv.style.opacity = 1;
	inputDiv.style.zIndex = 0;
	validationDiv.style.zIndex = 1;
} else {
	inputDiv.style.opacity = 1;
	inputDiv.style.zIndex = 1;
	validationDiv.style.zIndex = 0;
}
let html = hide("validation");
show ("validation", html);


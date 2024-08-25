const nameElement = document.getElementById("name");
const inputDiv = document.getElementById("input");
const inputTitle = document.getElementById("inputTitle");
const validationDiv = document.getElementById("validation");
const inputButton = document.getElementById("inputButton");

async function validate(name) {
	if (name != "" && name != null) {
		localStorage.setItem("newName", name);
		let response = await fetch("validate", {
			method: "POST",
			body: nameElement.value
		});
		let text = await response.text();
		console.log(text);
		if (text == "success") {
			inputDiv.style.opacity = 0;
			validationDiv.style.opacity = 1;
			inputDiv.style.zIndex = 0;
			validationDiv.style.zIndex = 1;
		} else {
			alert("That name is already taken");
		}
	} else {
		alert("Please input your name again");
	}
}
async function changeName() {
	validationDiv.style.opacity = 0;
	validationDiv.style.zIndex = 0;
	inputDiv.style.opacity = 1;
	inputDiv.style.zIndex = 1;
	inputButton.onclick = confirmChange;
}
async function confirmChange() {
	let name = localStorage.getItem("name");
	localStorage.setItem("newName", nameElement.value);
	let newName = localStorage.getItem("newName");
	if (newName != "" && newName != null) {
		console.log("confirming");
		let newName = localStorage.getItem("newName");
		let response = await fetch("changeName", {
			method: "PUT",
			body: `${name}&${newName}`
		});
		if (response.ok && newName != undefined && newName != "") {
			console.debug(localStorage.getItem("newName"));
			console.log(localStorage.getItem("name"));
			localStorage.setItem("name", localStorage.getItem("newName"));
		} else {
			console.log("no ok");
		}
		let text = await response.text();
	} else {
		alert("Please input your name again");
	}
	/*console.log(confirmation);
	if (confirmation == "pass") {
		let response = fetch("deleteName", {
			method: "POST",
			body: `${localStorage.getItem("name")}&${localStorage.getItem("newName")}`
		}).then((response) => {
			if (response.ok) {
				console.log(localStorage.getItem("newName"));
				console.log(localStorage.getItem("name"));
				localStorage.setItem("name", localStorage.getItem("newName"));
			} else {
				console.log("no ok");
			}
		});
	}*/
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
//let html = hide("validation");
//show ("validation", html);


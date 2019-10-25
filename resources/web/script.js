const tabs = {
    files: [
        document.querySelector("#files-tab"),
        document.querySelector("#files-form"),
    ],
    links: [
        document.querySelector("#links-tab"),
        document.querySelector("#links-form"),
    ],
    texts: [
        document.querySelector("#texts-tab"),
        document.querySelector("#texts-form"),
    ],
};

const inputs = {
    files: [
        document.querySelector("#files-url"),
        document.querySelector("#files-file"),
        document.querySelector("#files-submit"),
    ],
    links: [
        document.querySelector("#links-url"),
        document.querySelector("#links-forward"),
        document.querySelector("#links-submit"),
    ],
    texts: [
        document.querySelector("#texts-url"),
        document.querySelector("#texts-contents"),
        document.querySelector("#texts-submit"),
    ],
};

const randomUrl = () => {
    return Math.floor(Math.random() * 2147483647).toString(36);
}

for (const group in tabs) {
    tabs[group][0].onclick = () => {
        const active = document.querySelectorAll(".active");
        for (const el of active) {
            el.classList.remove("active");
        }
        for (const el of tabs[group]) {
            el.classList.add("active");
        }
    };
}

for (const group in inputs) {
    const submitButton = inputs[group][inputs[group].length - 1];

    const checkValidity = () => {
        submitButton.disabled = inputs[group].some(
            (input) => input.validity != undefined && !input.validity.valid
        );
    };

    const urlInput = inputs[group][0];
    urlInput.addEventListener("input", () => {
        urlInput.value = urlInput.value
            .replace(/[^0-9A-Za-z]/g, "")
            .toLowerCase();
        if (parseInt(urlInput.value, 36) > 2147483647) {
            urlInput.setCustomValidity(
                "Base 36 integer below or equal to zik0zj"
            );
        } else {
            urlInput.setCustomValidity("");
        }
    });
    urlInput.addEventListener("keyup", (e) => {
        if (e.code === "Space") {
            urlInput.value = randomUrl();
            checkValidity();
            e.preventDefault();
        }
    });

    for (const input of inputs[group].filter(
        (input) =>
            input instanceof HTMLInputElement ||
            input instanceof HTMLTextAreaElement
    )) {
        input.addEventListener("input", () => checkValidity());
        input.addEventListener("change", () => checkValidity());
    }

    const clearInputs = () => {
        for (const input of inputs[group].filter(
            (input) =>
                input instanceof HTMLInputElement ||
                input instanceof HTMLTextAreaElement
        )) {
            input.value = "";
        }
        submitButton.disabled = true;
    };

    let baseUrl = `${location.protocol}//${location.host}${location.pathname}`;
    if (!baseUrl.endsWith("/")) {
        baseUrl += "/";
    }

    if (group === "files") {
        const filesFileInput = inputs.files[1];
        const filesBrowseButton = document.querySelector("#files-browse");
        const filesValueInput = document.querySelector("#files-value");
        filesFileInput.addEventListener("change", () => {
            filesValueInput.value = filesFileInput.files[0].name || "";
        });
        filesBrowseButton.onclick = () => {
            filesFileInput.click();
        };
        filesValueInput.onfocus = (e) => {
            e.preventDefault();
            filesValueInput.blur();
            return false;
        };

        submitButton.addEventListener("click", () => {
            const file = filesFileInput.files[0];

            if (!file) {
                alert(new Error("No file selected"));
                return;
            }

            let fileReader = new FileReader();
            fileReader.onload = () => {
                const id = urlInput.value;
                const url = `${baseUrl}f/${id}`;

                const base64 = btoa(fileReader.result);
                const filename = file.name;
                let status;
                fetch(url, {
                    method: "PUT",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ base64, filename }),
                })
                    .then((response) => {
                        status = response.status;
                        return response.text();
                    })
                    .then((text) => {
                        if (status !== 201) {
                            throw new Error(text);
                        } else {
                            window.open(url, "_blank");
                            clearInputs();
                            filesValueInput.value = "";
                        }
                    })
                    .catch((error) => alert(error));
            };
            fileReader.readAsBinaryString(file);
        });
    } else if (group === "links") {
        submitButton.addEventListener("click", () => {
            const id = urlInput.value;
            const forward = inputs.links[1].value;

            const url = `${baseUrl}l/${id}`;
            let status;
            fetch(url, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ forward }),
            })
                .then((response) => {
                    status = response.status;
                    return response.text();
                })
                .then((text) => {
                    if (status !== 201) {
                        throw new Error(text);
                    } else {
                        window.open(url, "_blank");
                        clearInputs();
                    }
                })
                .catch((error) => alert(error));
        });
    } else if (group === "texts") {
        submitButton.addEventListener("click", () => {
            const id = urlInput.value;
            const contents = inputs.texts[1].value;

            const url = `${baseUrl}t/${id}`;
            let status;
            fetch(url, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ contents }),
            })
                .then((response) => {
                    status = response.status;
                    return response.text();
                })
                .then((text) => {
                    if (status !== 201) {
                        throw new Error(text);
                    } else {
                        window.open(url, "_blank");
                        clearInputs();
                    }
                })
                .catch((error) => alert(error));
        });
    }
}

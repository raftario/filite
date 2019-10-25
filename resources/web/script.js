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
    const checkValidity = () => {
        const submitButton = inputs[group][inputs[group].length - 1];
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

    for (const input of inputs[group].filter(
        (input) =>
            input instanceof HTMLInputElement ||
            input instanceof HTMLTextAreaElement
    )) {
        input.addEventListener("input", () => checkValidity());
        input.addEventListener("change", () => checkValidity());
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
    } else if (group === "links") {
    } else if (group === "texts") {
    }
}

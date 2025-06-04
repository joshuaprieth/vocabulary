document.addEventListener("DOMContentLoaded", () => {
    chrome.storage.local.get(["word", "definition"], (result) => {
        const output = document.getElementById('output');

        if (result.word && result.definition) {
            output.innerHTML = `<h1>${result.word}</h1>` + result["definition"];
        } else {
            output.textContent = "No text selected.";
        }
    });
});

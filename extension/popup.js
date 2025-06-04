document.addEventListener('DOMContentLoaded', () => {
    console.log("Popup loaded");
    chrome.storage.local.get(["word", "definition"], (result) => {
        console.log("Storage data:", result);

        const output = document.getElementById('output');

        if (result.word && result.definition) {
            output.innerHTML = result["definition"].join("<br>");
            chrome.storage.local.remove(["word", "definition"]);
        } else {
            output.textContent = "No text selected.";
        }
    });
});

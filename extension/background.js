// Create context menu item
chrome.contextMenus.create({
    id: "learnWord",
    title: 'Learn "%s"',
    contexts: ["selection"]
}, () => {
    if (chrome.runtime.lastError) {
        console.error("Error creating context menu:", chrome.runtime.lastError.message);
    }
});

// Handle context menu click
chrome.contextMenus.onClicked.addListener((info, tab) => {
    if (info.menuItemId === "learnWord" && info.selectionText) {
        let text = info.selectionText.trim();
        console.log("Selected text:", text);

        fetch("http://localhost:3000/api/v1/spanish/word/" + encodeURIComponent(text))
            .then((result) => result.json())
            .then((result) => {
                if (result.status === "ok") {
                    chrome.storage.local.set({
                        word: text,
                        definition: result["html"].join("<br>")
                    }, () => {
                        chrome.action.openPopup();
                    });
                } else if (result.status === "not found") {
                    chrome.storage.local.set({
                        word: text,
                        definition: "Not found in the dictionary."
                    }, () => {
                        chrome.action.openPopup();
                    });
                } else {
                    chrome.storage.local.set({
                        word: text,
                        definition: "An internal error occurred."
                    }, () => {
                        chrome.action.openPopup();
                    });
                }
            });

    }
});

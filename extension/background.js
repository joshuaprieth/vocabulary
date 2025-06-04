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
                console.log(result);

                chrome.storage.local.set({
                    word: text,
                    definition: result["html"]
                }, () => {
                    chrome.action.openPopup(); // Note: Chrome 88+ only
                });
            });

    }
});

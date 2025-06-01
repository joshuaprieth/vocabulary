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
        console.log("Selected text:", info.selectionText);

        chrome.storage.local.set({
            word: info.selectionText,
            definition: "test"
        }, () => {
            chrome.action.openPopup(); // Note: Chrome 88+ only
        });
    }
});

document.addEventListener("DOMContentLoaded", () => {
    chrome.storage.local.get(["word", "definition"], (result) => {
        updateContent(result);
    });
});

let prefetchLimit = 5;
function updateContent({ word, definition }) {
    const output = document.getElementById("output");

    if (word && definition) {
        output.innerHTML = `<h1>${word}</h1>` + definition;

        let prefetched = 0;
        for (let link of output.getElementsByTagName("a")) {
            if (link.rel === "mw:WikiLink" && link.href.endsWith("#Spanish") && !link.classList.contains("new") && link.title) {
                link.href = "#";
                let fetchPromise = prefetched < prefetchLimit ? fetchWord(link.title)
                    .then((result) => fetchResult = result) : null;
                let fetchResult = null;

                if (prefetched >= prefetchLimit) {
                    link.onmouseover = () => {
                        fetchPromise = fetchWord(link.title);
                    }
                } else {
                    prefetched += 1;
                }

                link.onclick = () => {
                    if (fetchResult) {
                        updateContent(fetchResult);
                    } else {
                        fetchPromise.then((result) => {
                            updateContent(result);
                        });
                    }
                }

            } else {
                link.removeAttribute("href");
            }
        }
    } else {
        output.textContent = "No text selected.";
    }
}

async function fetchWord(word) {
    let result = await fetch("http://localhost:3000/api/v1/spanish/word/" + encodeURIComponent(word));
    let json = await result.json();

    if (json.status === "ok") {
        return {
            word,
            definition: json["html"].join("<br>")
        };
    } else if (json.status === "not found") {
        return {
            word,
            definition: "Not found in the dictionary."
        };
    } else {
        return {
            word,
            definition: "An internal error occurred."
        };
    }
}

// One tab per window: a tab opened next to others moves to its own window
browser.tabs.onCreated.addListener(async (tab) => {
  const tabs = await browser.tabs.query({ windowId: tab.windowId });
  if (tabs.length > 1) {
    browser.windows.create({ tabId: tab.id });
  }
});

// The active tab's url at the start of the window title (`<url> | <title> — Mozilla Firefox`),
// where Sonamu reads it from to save the session
function showUrl(tab) {
  if (tab.active) {
    browser.windows.update(tab.windowId, { titlePreface: `${tab.url} | ` });
  }
}

browser.tabs.onUpdated.addListener((_id, _change, tab) => showUrl(tab), { properties: ["url"] });
browser.tabs.onActivated.addListener(async ({ tabId }) => showUrl(await browser.tabs.get(tabId)));
browser.tabs.onAttached.addListener(async (tabId) => showUrl(await browser.tabs.get(tabId)));
browser.tabs.query({ active: true }).then((tabs) => tabs.forEach(showUrl));

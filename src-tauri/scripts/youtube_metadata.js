// YouTube metadata extractor
window.__medyia_getMetadata = function () {
  const title =
    document.querySelector('#info h1 yt-formatted-string')?.textContent?.trim() ||
    document.querySelector('h1.ytd-watch-metadata yt-formatted-string')?.textContent?.trim() ||
    document.querySelector('#title h1 yt-formatted-string')?.textContent?.trim() ||
    document.title.replace(' - YouTube', '');

  const artist =
    document.querySelector('#owner ytd-channel-name yt-formatted-string a')?.textContent?.trim() ||
    document.querySelector('#channel-name a')?.textContent?.trim() ||
    document.querySelector('ytd-channel-name a')?.textContent?.trim() ||
    null;

  const artworkUrl =
    document.querySelector('meta[property="og:image"]')?.content ||
    null;

  return { title, artist, artworkUrl };
};

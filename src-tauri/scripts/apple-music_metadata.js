// Apple Music metadata extractor
window.__medyia_getMetadata = function () {
  const title =
    document.querySelector('.web-chrome-playback-lcd__song-name-scroll-inner')?.textContent?.trim() ||
    document.querySelector('[class*="playback-lcd"] [class*="song-name"]')?.textContent?.trim() ||
    document.title;

  const artist =
    document.querySelector('.web-chrome-playback-lcd__sub-copy-scroll-inner')?.textContent?.trim() ||
    document.querySelector('[class*="playback-lcd"] [class*="sub-copy"]')?.textContent?.trim() ||
    null;

  const artworkUrl =
    document.querySelector('.web-chrome-playback-lcd__artwork source')?.srcset?.split(' ')[0] ||
    document.querySelector('[class*="playback-lcd"] picture img')?.src ||
    null;

  return { title, artist, artworkUrl };
};

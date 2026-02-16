// SoundCloud metadata extractor
window.__medya_getMetadata = function () {
  const titleEl = document.querySelector('.playbackSoundBadge__titleLink');
  const title = titleEl?.getAttribute('title') || titleEl?.textContent?.trim() || document.title;

  const artist =
    document.querySelector('.playbackSoundBadge__lightLink')?.getAttribute('title') ||
    document.querySelector('.playbackSoundBadge__lightLink')?.textContent?.trim() ||
    null;

  const artworkUrl =
    document.querySelector('.playbackSoundBadge__avatar .image span')?.style?.backgroundImage?.replace(/url\(["']?|["']?\)/g, '') ||
    null;

  return { title, artist, artworkUrl };
};

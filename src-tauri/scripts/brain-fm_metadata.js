// Brain.fm metadata extractor
window.__medyia_getMetadata = function () {
  const title =
    document.querySelector('[data-testid="currentTrackTitle"]')?.textContent?.trim() ||
    document.title;

  // Brain.fm surfaces genre rather than a traditional artist name
  const artist =
    document.querySelector('[data-testid="trackGenre"]')?.textContent?.trim() || null;

  const artworkUrl =
    document.querySelector('[data-testid="currentTrackInformationCard"] img')?.src || null;

  return { title, artist, artworkUrl };
};

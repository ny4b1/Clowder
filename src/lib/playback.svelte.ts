class PlaybackMemory {
  volume = $state(1);
  muted = $state(false);
}

export const playbackMemory = new PlaybackMemory();

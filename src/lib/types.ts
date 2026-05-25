export type PostFile = {
  ext: string;
  width?: number | null;
  height?: number | null;
  url?: string | null;
};

export type PostImage = {
  url?: string | null;
};

export type PostTags = {
  artist?: string[];
  copyright?: string[];
  character?: string[];
  species?: string[];
  general?: string[];
  meta?: string[];
  lore?: string[];
};

export type PostScore = {
  up?: number;
  down?: number;
  total?: number;
};

export type Post = {
  id: number;
  file: PostFile;
  preview?: PostImage;
  sample?: PostImage;
  tags?: PostTags;
  is_favorited?: boolean;
  score?: PostScore;
  fav_count?: number;
  comment_count?: number;
};

export type Comment = {
  id: number;
  created_at?: string | null;
  updated_at?: string | null;
  post_id: number;
  creator_id?: number | null;
  creator_name?: string | null;
  updater_id?: number | null;
  body: string;
  score?: number;
  is_hidden?: boolean;
  is_sticky?: boolean;
  do_not_bump_post?: boolean;
};

export type CommentState = {
  items: Comment[];
  loading: boolean;
  error: string | null;
  body: string;
  submitting: boolean;
  submitError: string | null;
  hiding: Record<number, boolean>;
};

export type SearchResponse = {
  posts: Post[];
};

export type TagSuggestion = {
  id: number;
  name: string;
  post_count: number;
  category: number;
  insert?: string;
};

export type AccountResponse = {
  username: string | null;
};

export type Preset = {
  label: string;
  value: string;
  requiresAccount?: boolean;
};

export type SortMode = "latest" | "popular";

export type OriginalViewer = {
  post: Post;
  dataUrl: string | null;
  loading: boolean;
  error: string | null;
};

export type DownloadSettings = {
  directory: string | null;
  filename_template: string;
};

export type PlaybackSettings = {
  autoplay: boolean;
  remember_volume: boolean;
  video_chunk_mb: number;
};

export type Theme = "system" | "dark" | "light";

export type MotionPreference = "system" | "always" | "never";

export type AppearanceSettings = {
  theme: Theme;
  motion: MotionPreference;
  grid_min_tile_px: number;
};

export type Settings = {
  downloads: DownloadSettings;
  playback: PlaybackSettings;
  appearance: AppearanceSettings;
};

export type SettingsSection =
  | "account"
  | "downloads"
  | "playback"
  | "appearance"
  | "about";

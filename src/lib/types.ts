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
};

export type SearchResponse = {
  posts: Post[];
  ech_enabled: boolean;
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

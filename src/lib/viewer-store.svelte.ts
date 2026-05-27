import { createComment, fetchComments, hideComment, mediaUrl, originalUrl } from "./e621";
import { errMsg } from "./errors";
import type { CommentState, OriginalViewer, Post } from "./types";

const emptyComments: CommentState = {
  items: [],
  loading: false,
  error: null,
  body: "",
  submitting: false,
  submitError: null,
  hiding: {},
};

class ViewerStore {
  viewer = $state<OriginalViewer | null>(null);
  imageOnly = $state(false);
  comments = $state<CommentState>({ ...emptyComments });

  async open(post: Post, historyMode: "push" | "replace" | "skip" = "push") {
    const url = originalUrl(post);
    this.imageOnly = false;
    this.comments = { ...emptyComments, loading: true };
    this.viewer = {
      post,
      dataUrl: null,
      loading: !!url,
      error: url ? null : "original file is unavailable",
    };
    if (historyMode === "push") {
      history.pushState({ viewer: post.id }, "", "");
    } else if (historyMode === "replace") {
      history.replaceState({ viewer: post.id }, "", "");
    }

    void this.loadComments(post.id);

    if (!url) return;

    try {
      const result = await mediaUrl(url);
      if (this.viewer?.post.id === post.id) {
        this.viewer = { ...this.viewer, dataUrl: result, loading: false, error: null };
      }
    } catch (error) {
      if (this.viewer?.post.id === post.id) {
        this.viewer = { ...this.viewer, dataUrl: null, loading: false, error: errMsg(error) };
      }
    }
  }

  close(fromHistory = false) {
    this.viewer = null;
    this.imageOnly = false;
    this.comments = { ...emptyComments };
    if (!fromHistory && history.state?.viewer) {
      history.back();
    }
  }

  toggleImageOnly() {
    this.imageOnly = !this.imageOnly;
  }

  setImageOnly(value: boolean) {
    this.imageOnly = value;
  }

  updateViewerPost(id: number, patch: Partial<Post>) {
    if (!this.viewer || this.viewer.post.id !== id) return;
    this.viewer = { ...this.viewer, post: { ...this.viewer.post, ...patch } };
  }

  replaceViewerPost(post: Post) {
    if (!this.viewer || this.viewer.post.id !== post.id) return;
    this.viewer = { ...this.viewer, post };
  }

  async loadComments(postId: number) {
    this.comments = { ...this.comments, loading: true, error: null };
    try {
      const items = await fetchComments(postId);
      if (this.viewer?.post.id === postId) {
        this.comments = { ...this.comments, items, loading: false, error: null };
      }
    } catch (error) {
      if (this.viewer?.post.id === postId) {
        this.comments = { ...this.comments, items: [], loading: false, error: errMsg(error) };
      }
    }
  }

  setCommentBody(value: string) {
    this.comments = { ...this.comments, body: value, submitError: null };
  }

  async submitComment(post: Post): Promise<number | null> {
    const body = this.comments.body.trim();
    if (!body || this.comments.submitting) return null;

    this.comments = { ...this.comments, submitting: true, submitError: null };

    try {
      const created = await createComment(post.id, body);
      if (this.viewer?.post.id !== post.id) return null;
      const nextCount = (this.viewer.post.comment_count ?? this.comments.items.length) + 1;
      this.viewer = {
        ...this.viewer,
        post: { ...this.viewer.post, comment_count: nextCount },
      };
      this.comments = {
        ...this.comments,
        items: [...this.comments.items, created],
        body: "",
        submitting: false,
        submitError: null,
      };
      return nextCount;
    } catch (error) {
      this.comments = { ...this.comments, submitting: false, submitError: errMsg(error) };
      return null;
    }
  }

  async hideOwnComment(commentId: number): Promise<void> {
    if (this.comments.hiding[commentId]) return;
    this.comments = {
      ...this.comments,
      submitError: null,
      hiding: { ...this.comments.hiding, [commentId]: true },
    };
    try {
      const hidden = await hideComment(commentId);
      this.comments = {
        ...this.comments,
        items: this.comments.items.map((comment) =>
          comment.id === commentId ? { ...comment, ...hidden, is_hidden: true } : comment,
        ),
        hiding: { ...this.comments.hiding, [commentId]: false },
      };
    } catch (error) {
      this.comments = {
        ...this.comments,
        submitError: errMsg(error),
        hiding: { ...this.comments.hiding, [commentId]: false },
      };
    }
  }
}

export const viewerStore = new ViewerStore();

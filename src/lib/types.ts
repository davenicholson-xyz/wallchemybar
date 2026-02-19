export interface Tag {
    id: number;
    name: string;
}

export interface Wallpaper {
    id: string;
    url: string;
    path: string;
    thumbs: { large: string; original: string; small: string };
    resolution: string;
    tags?: Tag[];
}

export interface Collection {
    id: number;
    label: string;
    views: number;
    public: number;
    count: number;
}

export type View =
    | { kind: "search"; sorting: string }
    | { kind: "collections" }
    | { kind: "collection"; id: number }
    | { kind: "query"; query: string }
    | { kind: "history" }
    | { kind: "queue" }
    | { kind: "settings" };

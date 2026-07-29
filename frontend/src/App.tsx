import { FormEvent, Fragment, useCallback, useEffect, useMemo, useState } from "react";
import {
  CaptureNow,
  GetCapture,
  GetStatus,
  GetStorageStats,
  PauseCapture,
  RefreshStatus,
  RequestAccessibilityPermission,
  RequestScreenPermission,
  ResumeCapture,
  Search,
} from "../wailsjs/go/main/App";
import { EventsOff, EventsOn } from "../wailsjs/runtime/runtime";

type PermissionState = "granted" | "required" | "unknown" | "unsupported";

type AppStatus = {
  state: string;
  screenPermission: PermissionState;
  accessibilityPermission: PermissionState;
  lastCapturedAt: number;
  captureCount: number;
  lastError: string;
};

type HighlightRange = {
  start: number;
  end: number;
};

type TextBlock = {
  id: number;
  captureId: number;
  text: string;
  confidence: number;
  x: number;
  y: number;
  width: number;
  height: number;
};

type SearchResult = {
  id: number;
  capturedAt: number;
  displayIndex: number;
  width: number;
  height: number;
  appName: string;
  bundleId: string;
  windowTitle: string;
  snippet: string;
  highlights: HighlightRange[];
  matchingBoxes: TextBlock[];
  ocrStatus: string;
};

type SearchPage = {
  results: SearchResult[];
  nextOffset: number;
  hasMore: boolean;
};

type CaptureDetail = {
  id: number;
  capturedAt: number;
  displayIndex: number;
  width: number;
  height: number;
  appName: string;
  bundleId: string;
  processId: number;
  windowTitle: string;
  windowX: number;
  windowY: number;
  windowWidth: number;
  windowHeight: number;
  ocrStatus: string;
  ocrError: string;
  blocks: TextBlock[];
};

type DailyStorage = {
  date: string;
  imageBytes: number;
  count: number;
};

type StorageStats = {
  imageBytes: number;
  databaseBytes: number;
  captureCount: number;
  daily: DailyStorage[];
};

const emptyStatus: AppStatus = {
  state: "loading",
  screenPermission: "unknown",
  accessibilityPermission: "unknown",
  lastCapturedAt: 0,
  captureCount: 0,
  lastError: "",
};

const emptyStorage: StorageStats = {
  imageBytes: 0,
  databaseBytes: 0,
  captureCount: 0,
  daily: [],
};

function App() {
  const [query, setQuery] = useState("");
  const [activeQuery, setActiveQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [nextOffset, setNextOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [status, setStatus] = useState<AppStatus>(emptyStatus);
  const [storage, setStorage] = useState<StorageStats>(emptyStorage);
  const [selected, setSelected] = useState<SearchResult | null>(null);
  const [detail, setDetail] = useState<CaptureDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState("");

  const loadStorage = useCallback(async () => {
    try {
      setStorage((await GetStorageStats()) as StorageStats);
    } catch (value) {
      setError(readError(value));
    }
  }, []);

  const runSearch = useCallback(async (value: string, offset = 0) => {
    if (offset === 0) {
      setLoading(true);
    } else {
      setLoadingMore(true);
    }
    setError("");
    try {
      const page = (await Search({
        query: value,
        offset,
        limit: 40,
      })) as SearchPage;
      setResults((current) => offset === 0 ? page.results : [...current, ...page.results]);
      setNextOffset(page.nextOffset);
      setHasMore(page.hasMore);
      setActiveQuery(value);
    } catch (value) {
      setError(readError(value));
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, []);

  useEffect(() => {
    Promise.all([
      GetStatus().then((value) => setStatus(value as AppStatus)),
      loadStorage(),
      runSearch(""),
    ]).catch((value) => setError(readError(value)));

    EventsOn("status", (value: AppStatus) => {
      setStatus(value);
      if (value.lastCapturedAt > 0) {
        void loadStorage();
      }
    });
    const refresh = window.setInterval(() => {
      void RefreshStatus().then((value) => setStatus(value as AppStatus));
      void loadStorage();
    }, 60_000);

    return () => {
      window.clearInterval(refresh);
      EventsOff("status");
    };
  }, [loadStorage, runSearch]);

  useEffect(() => {
    if (selected === null) {
      setDetail(null);
      return;
    }
    GetCapture(selected.id)
      .then((value) => setDetail(value as CaptureDetail))
      .catch((value) => setError(readError(value)));
  }, [selected]);

  const grouped = useMemo(() => {
    const groups = new Map<string, SearchResult[]>();
    for (const result of results) {
      const day = dayLabel(result.capturedAt);
      groups.set(day, [...(groups.get(day) ?? []), result]);
    }
    return [...groups.entries()];
  }, [results]);

  function submit(event: FormEvent) {
    event.preventDefault();
    void runSearch(query.trim());
  }

  async function askScreenPermission() {
    setStatus((await RequestScreenPermission()) as AppStatus);
  }

  async function askAccessibilityPermission() {
    setStatus((await RequestAccessibilityPermission()) as AppStatus);
  }

  async function toggleCapture() {
    if (status.state === "running" || status.state === "waiting") {
      setStatus((await PauseCapture()) as AppStatus);
    } else {
      setStatus((await ResumeCapture()) as AppStatus);
    }
  }

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandMark" aria-hidden="true">
            <span />
            <span />
          </div>
          <div>
            <h1>Dejavu</h1>
            <p>Your visual memory</p>
          </div>
        </div>

        <section className="statusCard">
          <div className="statusHeading">
            <span className={`statusDot ${status.state}`} />
            <div>
              <strong>{statusLabel(status)}</strong>
              <span>{lastCaptureLabel(status.lastCapturedAt)}</span>
            </div>
          </div>
          <div className="captureActions">
            <button className="quietButton" onClick={() => void toggleCapture()}>
              {status.state === "running" || status.state === "waiting" ? "Pause" : "Resume"}
            </button>
            <button
              className="iconButton"
              onClick={() => void CaptureNow()}
              disabled={status.state !== "running"}
              title="Capture now"
              aria-label="Capture now"
            >
              <CaptureIcon />
            </button>
          </div>
        </section>

        {(status.screenPermission !== "granted" ||
          status.accessibilityPermission !== "granted") && (
          <section className="permissionCard">
            <p className="eyebrow">Permissions</p>
            {status.screenPermission !== "granted" && (
              <PermissionRow
                label="Screen recording"
                detail="Needed to save screenshots"
                onClick={askScreenPermission}
              />
            )}
            {status.accessibilityPermission !== "granted" && (
              <PermissionRow
                label="Window details"
                detail="Adds the active app and title"
                onClick={askAccessibilityPermission}
              />
            )}
          </section>
        )}

        <section className="storageCard">
          <div className="storageTitle">
            <p className="eyebrow">Local storage</p>
            <span>{storage.captureCount.toLocaleString()} images</span>
          </div>
          <StorageBar storage={storage} />
          <div className="storageTotals">
            <span>Images {formatBytes(storage.imageBytes)}</span>
            <span>Database {formatBytes(storage.databaseBytes)}</span>
          </div>
          <div className="dailyList">
            {storage.daily.slice(0, 5).map((day) => (
              <div key={day.date}>
                <span>{shortDate(day.date)}</span>
                <span>{day.count} · {formatBytes(day.imageBytes)}</span>
              </div>
            ))}
          </div>
          <p className="storageNote">Stored only in ~/.dejavu</p>
        </section>

        <p className="sidebarFoot">Private by default</p>
      </aside>

      <main className="main">
        <header className="topbar">
          <form className="searchForm" onSubmit={submit}>
            <SearchIcon />
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search what you saw"
              aria-label="Search screenshots"
            />
            {query && (
              <button
                type="button"
                className="clearButton"
                onClick={() => setQuery("")}
                aria-label="Clear search"
              >
                ×
              </button>
            )}
            <kbd>return</kbd>
          </form>
        </header>

        <div className="content">
          <div className="pageIntro">
            <div>
              <p className="eyebrow">{activeQuery ? "Search results" : "Timeline"}</p>
              <h2>{activeQuery ? `Matches for “${activeQuery}”` : "What you saw"}</h2>
            </div>
            <span className="resultCount">{results.length} shown</span>
          </div>

          {error && <div className="errorBanner">{error}</div>}
          {status.lastError && <div className="errorBanner">{status.lastError}</div>}

          {loading ? (
            <LoadingGrid />
          ) : results.length === 0 ? (
            <EmptyState searched={activeQuery.length > 0} />
          ) : (
            <div className="timeline">
              {grouped.map(([day, items]) => (
                <section className="dayGroup" key={day}>
                  <div className="dayLabel">
                    <span>{day}</span>
                    <i />
                  </div>
                  <div className="captureGrid">
                    {items.map((result) => (
                      <CaptureCard
                        key={result.id}
                        result={result}
                        onClick={() => setSelected(result)}
                      />
                    ))}
                  </div>
                </section>
              ))}
              {hasMore && (
                <button
                  className="loadButton"
                  disabled={loadingMore}
                  onClick={() => void runSearch(activeQuery, nextOffset)}
                >
                  {loadingMore ? "Loading…" : "Show more"}
                </button>
              )}
            </div>
          )}
        </div>
      </main>

      {selected && (
        <DetailPanel
          result={selected}
          detail={detail}
          onClose={() => setSelected(null)}
        />
      )}
    </div>
  );
}

function PermissionRow(props: {
  label: string;
  detail: string;
  onClick: () => Promise<void>;
}) {
  return (
    <div className="permissionRow">
      <div>
        <strong>{props.label}</strong>
        <span>{props.detail}</span>
      </div>
      <button onClick={() => void props.onClick()}>Allow</button>
    </div>
  );
}

function StorageBar({ storage }: { storage: StorageStats }) {
  const database = storage.databaseBytes;
  const total = storage.imageBytes + database;
  const databaseWidth = total === 0 || database === 0
    ? 0
    : Math.max(4, database / total * 100);
  const imageWidth = total === 0 ? 0 : 100 - databaseWidth;
  return (
    <div className="storageBar" aria-label={`${formatBytes(total)} used`}>
      <span style={{ width: `${imageWidth}%` }} />
      <i style={{ width: `${databaseWidth}%` }} />
    </div>
  );
}

function CaptureCard({
  result,
  onClick,
}: {
  result: SearchResult;
  onClick: () => void;
}) {
  return (
    <button className="captureCard" onClick={onClick}>
      <div className="thumb">
        <img src={`/capture/${result.id}/thumbnail`} alt="" loading="lazy" />
        <span className="timeBadge">{timeLabel(result.capturedAt)}</span>
        {result.displayIndex > 0 && (
          <span className="displayBadge">Display {result.displayIndex + 1}</span>
        )}
      </div>
      <div className="cardBody">
        <div className="appLine">
          <span className="appGlyph">{initial(result.appName)}</span>
          <strong>{result.appName || "Unknown app"}</strong>
        </div>
        <p className="windowTitle">{result.windowTitle || "Untitled window"}</p>
        {result.snippet && (
          <p className="snippet">
            <HighlightedText text={result.snippet} ranges={result.highlights} />
          </p>
        )}
      </div>
    </button>
  );
}

function HighlightedText({
  text,
  ranges,
}: {
  text: string;
  ranges: HighlightRange[];
}) {
  const letters = Array.from(text);
  const parts: React.ReactNode[] = [];
  let cursor = 0;
  for (const range of ranges) {
    if (range.start > cursor) {
      parts.push(<Fragment key={`plain${cursor}`}>{letters.slice(cursor, range.start).join("")}</Fragment>);
    }
    parts.push(
      <mark key={`mark${range.start}`}>
        {letters.slice(range.start, range.end).join("")}
      </mark>,
    );
    cursor = Math.max(cursor, range.end);
  }
  if (cursor < letters.length) {
    parts.push(<Fragment key={`plain${cursor}`}>{letters.slice(cursor).join("")}</Fragment>);
  }
  return <>{parts}</>;
}

function DetailPanel({
  result,
  detail,
  onClose,
}: {
  result: SearchResult;
  detail: CaptureDetail | null;
  onClose: () => void;
}) {
  const boxes = result.matchingBoxes ?? [];
  return (
    <div className="detailShade" onMouseDown={onClose}>
      <section
        className="detailPanel"
        onMouseDown={(event) => event.stopPropagation()}
        aria-label="Capture detail"
      >
        <header className="detailHeader">
          <div>
            <p className="eyebrow">{fullDate(result.capturedAt)}</p>
            <h3>{result.windowTitle || result.appName || "Captured screen"}</h3>
          </div>
          <button onClick={onClose} aria-label="Close detail">×</button>
        </header>
        <DetailImage key={result.id} result={result} boxes={boxes} />
        <footer className="detailMeta">
          <div>
            <span>Application</span>
            <strong>{result.appName || "Unknown"}</strong>
          </div>
          <div>
            <span>Display</span>
            <strong>{result.displayIndex + 1}</strong>
          </div>
          <div>
            <span>Image</span>
            <strong>{result.width} × {result.height}</strong>
          </div>
          <div>
            <span>OCR</span>
            <strong>{detail?.ocrStatus ?? result.ocrStatus}</strong>
          </div>
        </footer>
        {detail?.ocrError && <p className="detailError">{detail.ocrError}</p>}
      </section>
    </div>
  );
}

function DetailImage({
  result,
  boxes,
}: {
  result: SearchResult;
  boxes: TextBlock[];
}) {
  const [ready, setReady] = useState(false);
  const [failed, setFailed] = useState(false);
  const imageWidth = Math.max(1, result.width);
  const imageHeight = Math.max(1, result.height);
  const imageRatio = imageWidth / imageHeight;

  async function finishLoad(element: HTMLImageElement) {
    try {
      await element.decode();
    } catch {
      if (!element.complete || element.naturalWidth === 0) {
        setFailed(true);
        return;
      }
    }
    setReady(true);
  }

  return (
    <div
      className="detailImage"
      aria-busy={!ready && !failed}
      style={{
        aspectRatio: `${imageWidth} / ${imageHeight}`,
        width: `min(calc(100% - 42px), calc(67vh * ${imageRatio}))`,
      }}
    >
      <img
        className={`detailPreview ${ready ? "isHidden" : ""}`}
        src={`/capture/${result.id}/thumbnail`}
        alt=""
      />
      <img
        className={`detailFull ${ready ? "isReady" : ""}`}
        src={`/capture/${result.id}/image`}
        alt="Captured screen"
        onLoad={(event) => void finishLoad(event.currentTarget)}
        onError={() => setFailed(true)}
      />
      {!ready && !failed && (
        <div className="imageLoading" role="status">
          <span />
          <p>Loading full image</p>
        </div>
      )}
      {failed && (
        <div className="imageFailure" role="alert">
          <strong>Could not load image</strong>
          <span>The screenshot file is unavailable.</span>
        </div>
      )}
      {ready && boxes.map((box, index) => (
        <span
          className="matchBox"
          key={`${box.id}${index}`}
          style={{
            left: `${box.x * 100}%`,
            top: `${box.y * 100}%`,
            width: `${box.width * 100}%`,
            height: `${box.height * 100}%`,
          }}
        />
      ))}
    </div>
  );
}

function LoadingGrid() {
  return (
    <div className="loadingGrid">
      {[0, 1, 2, 3, 4, 5].map((value) => <span key={value} />)}
    </div>
  );
}

function EmptyState({ searched }: { searched: boolean }) {
  return (
    <div className="emptyState">
      <div className="emptyIcon"><SearchIcon /></div>
      <h3>{searched ? "No matching moments" : "Your timeline is ready"}</h3>
      <p>
        {searched
          ? "Try fewer words or a different phrase."
          : "Once screen access is allowed, new screenshots appear here."}
      </p>
    </div>
  );
}

function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="11" cy="11" r="6.5" />
      <path d="m16 16 4 4" />
    </svg>
  );
}

function CaptureIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <rect x="4" y="6" width="16" height="12" rx="2" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

function statusLabel(status: AppStatus) {
  if (status.screenPermission !== "granted") {
    return "Waiting for access";
  }
  if (status.state === "running") {
    return "Recording";
  }
  return "Paused";
}

function lastCaptureLabel(value: number) {
  if (!value) {
    return "No captures yet";
  }
  return `Last saved ${relativeTime(value)}`;
}

function relativeTime(value: number) {
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - value));
  if (seconds < 60) {
    return "just now";
  }
  if (seconds < 3600) {
    return `${Math.floor(seconds / 60)}m ago`;
  }
  return `${Math.floor(seconds / 3600)}h ago`;
}

function timeLabel(value: number) {
  return new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

function dayLabel(value: number) {
  const date = new Date(value * 1000);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);
  if (date.toDateString() === today.toDateString()) {
    return "Today";
  }
  if (date.toDateString() === yesterday.toDateString()) {
    return "Yesterday";
  }
  return new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "short",
    day: "numeric",
  }).format(date);
}

function fullDate(value: number) {
  return new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

function shortDate(value: string) {
  const [year, month, day] = value.split("-").map(Number);
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
  }).format(new Date(year, month - 1, day));
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  const units = ["KB", "MB", "GB", "TB"];
  let size = value / 1024;
  let index = 0;
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024;
    index++;
  }
  return `${size >= 10 ? size.toFixed(0) : size.toFixed(1)} ${units[index]}`;
}

function initial(value: string) {
  return Array.from(value.trim())[0]?.toUpperCase() ?? "•";
}

function readError(value: unknown) {
  if (value instanceof Error) {
    return value.message;
  }
  return String(value);
}

export default App;

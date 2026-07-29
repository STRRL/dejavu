export namespace model {

	export class AppStatus {
	    state: string;
	    screenPermission: string;
	    accessibilityPermission: string;
	    lastCapturedAt: number;
	    captureCount: number;
	    lastError: string;

	    static createFrom(source: any = {}) {
	        return new AppStatus(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.state = source["state"];
	        this.screenPermission = source["screenPermission"];
	        this.accessibilityPermission = source["accessibilityPermission"];
	        this.lastCapturedAt = source["lastCapturedAt"];
	        this.captureCount = source["captureCount"];
	        this.lastError = source["lastError"];
	    }
	}
	export class TextBlock {
	    id: number;
	    captureId: number;
	    text: string;
	    confidence: number;
	    x: number;
	    y: number;
	    width: number;
	    height: number;

	    static createFrom(source: any = {}) {
	        return new TextBlock(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.captureId = source["captureId"];
	        this.text = source["text"];
	        this.confidence = source["confidence"];
	        this.x = source["x"];
	        this.y = source["y"];
	        this.width = source["width"];
	        this.height = source["height"];
	    }
	}
	export class CaptureDetail {
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

	    static createFrom(source: any = {}) {
	        return new CaptureDetail(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.capturedAt = source["capturedAt"];
	        this.displayIndex = source["displayIndex"];
	        this.width = source["width"];
	        this.height = source["height"];
	        this.appName = source["appName"];
	        this.bundleId = source["bundleId"];
	        this.processId = source["processId"];
	        this.windowTitle = source["windowTitle"];
	        this.windowX = source["windowX"];
	        this.windowY = source["windowY"];
	        this.windowWidth = source["windowWidth"];
	        this.windowHeight = source["windowHeight"];
	        this.ocrStatus = source["ocrStatus"];
	        this.ocrError = source["ocrError"];
	        this.blocks = this.convertValues(source["blocks"], TextBlock);
	    }

		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class DailyStorage {
	    date: string;
	    imageBytes: number;
	    count: number;

	    static createFrom(source: any = {}) {
	        return new DailyStorage(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.date = source["date"];
	        this.imageBytes = source["imageBytes"];
	        this.count = source["count"];
	    }
	}
	export class HighlightRange {
	    start: number;
	    end: number;

	    static createFrom(source: any = {}) {
	        return new HighlightRange(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.start = source["start"];
	        this.end = source["end"];
	    }
	}
	export class SearchResult {
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

	    static createFrom(source: any = {}) {
	        return new SearchResult(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.capturedAt = source["capturedAt"];
	        this.displayIndex = source["displayIndex"];
	        this.width = source["width"];
	        this.height = source["height"];
	        this.appName = source["appName"];
	        this.bundleId = source["bundleId"];
	        this.windowTitle = source["windowTitle"];
	        this.snippet = source["snippet"];
	        this.highlights = this.convertValues(source["highlights"], HighlightRange);
	        this.matchingBoxes = this.convertValues(source["matchingBoxes"], TextBlock);
	        this.ocrStatus = source["ocrStatus"];
	    }

		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class SearchPage {
	    results: SearchResult[];
	    nextOffset: number;
	    hasMore: boolean;

	    static createFrom(source: any = {}) {
	        return new SearchPage(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.results = this.convertValues(source["results"], SearchResult);
	        this.nextOffset = source["nextOffset"];
	        this.hasMore = source["hasMore"];
	    }

		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class SearchRequest {
	    query: string;
	    offset: number;
	    limit: number;

	    static createFrom(source: any = {}) {
	        return new SearchRequest(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.query = source["query"];
	        this.offset = source["offset"];
	        this.limit = source["limit"];
	    }
	}

	export class StorageStats {
	    imageBytes: number;
	    databaseBytes: number;
	    captureCount: number;
	    daily: DailyStorage[];

	    static createFrom(source: any = {}) {
	        return new StorageStats(source);
	    }

	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.imageBytes = source["imageBytes"];
	        this.databaseBytes = source["databaseBytes"];
	        this.captureCount = source["captureCount"];
	        this.daily = this.convertValues(source["daily"], DailyStorage);
	    }

		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

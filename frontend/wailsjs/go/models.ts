export namespace utils {
	
	export class DiskInfo {
	    device: string;
	    mountpoint: string;
	    total: number;
	    free: number;
	    used: number;
	    usedPercent: number;
	
	    static createFrom(source: any = {}) {
	        return new DiskInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.device = source["device"];
	        this.mountpoint = source["mountpoint"];
	        this.total = source["total"];
	        this.free = source["free"];
	        this.used = source["used"];
	        this.usedPercent = source["usedPercent"];
	    }
	}
	export class SystemStats {
	    cpuPercent: number;
	    ramTotal: number;
	    ramUsed: number;
	    ramPercent: number;
	    gpuInfo: string;
	    gpuPercent: number;
	    disks: DiskInfo[];
	
	    static createFrom(source: any = {}) {
	        return new SystemStats(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.cpuPercent = source["cpuPercent"];
	        this.ramTotal = source["ramTotal"];
	        this.ramUsed = source["ramUsed"];
	        this.ramPercent = source["ramPercent"];
	        this.gpuInfo = source["gpuInfo"];
	        this.gpuPercent = source["gpuPercent"];
	        this.disks = this.convertValues(source["disks"], DiskInfo);
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


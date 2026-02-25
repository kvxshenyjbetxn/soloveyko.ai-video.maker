import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';

interface LogEntry {
    id: string;
    timestamp: Date;
    level: string;
    message: string;
    taskLabel?: string;
    taskId?: string;
}

interface VirtualLogListProps {
    logs: LogEntry[];
    rowHeight?: number;
    className?: string;
    renderRow: (log: LogEntry) => React.ReactNode;
}

export const VirtualLogList: React.FC<VirtualLogListProps> = ({
    logs,
    rowHeight = 24,
    className = "",
    renderRow
}) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const [scrollTop, setScrollTop] = useState(0);
    const [containerHeight, setContainerHeight] = useState(0);

    // Update container height on resize
    useEffect(() => {
        const updateHeight = () => {
            if (containerRef.current) {
                setContainerHeight(containerRef.current.clientHeight);
            }
        };

        updateHeight();
        window.addEventListener('resize', updateHeight);

        // Also use ResizeObserver for more accuracy in dynamic layouts
        const observer = new ResizeObserver(updateHeight);
        if (containerRef.current) observer.observe(containerRef.current);

        return () => {
            window.removeEventListener('resize', updateHeight);
            observer.disconnect();
        };
    }, []);

    const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
        setScrollTop(e.currentTarget.scrollTop);
    }, []);

    const { startIndex, endIndex, translateY } = useMemo(() => {
        // Basic indices calculation
        const start = Math.floor(scrollTop / rowHeight);
        const count = Math.ceil(containerHeight / rowHeight);

        // Add buffer
        const buffer = 10;
        const bufferedStart = Math.max(0, start - buffer);
        const bufferedEnd = Math.min(logs.length, start + count + buffer);

        return {
            startIndex: bufferedStart,
            endIndex: bufferedEnd,
            translateY: bufferedStart * rowHeight
        };
    }, [scrollTop, containerHeight, logs.length, rowHeight]);

    const totalHeight = logs.length * rowHeight;
    const visibleLogs = useMemo(() => logs.slice(startIndex, endIndex), [logs, startIndex, endIndex]);

    return (
        <div
            ref={containerRef}
            className={`virtual-log-container premium-scrollbar ${className}`}
            onScroll={handleScroll}
            style={{
                position: 'relative',
                overflowY: 'auto',
                height: '100%',
                width: '100%'
            }}
        >
            <div style={{ height: totalHeight, width: '100%', position: 'relative' }}>
                <div style={{
                    transform: `translateY(${translateY}px)`,
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    right: 0
                }}>
                    {visibleLogs.map(renderRow)}
                </div>
            </div>
        </div>
    );
};

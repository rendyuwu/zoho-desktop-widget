import { LoadingSkeleton } from "@gio/bigsu-ui";

function LoadingState() {
  return (
    <div
      className="flex flex-1 flex-col gap-4 p-3"
      aria-busy="true"
      role="status"
    >
      <div className="grid grid-cols-2 gap-2">
        {[0, 1, 2, 3].map((i) => (
          <div
            key={i}
            className="flex flex-col gap-2 rounded-lg border border-border-default p-3"
          >
            <LoadingSkeleton className="h-3 w-20" />
            <LoadingSkeleton className="h-6 w-12" />
            <LoadingSkeleton className="h-3 w-16" />
          </div>
        ))}
      </div>
      <div className="flex flex-col gap-2">
        {[0, 1, 2].map((i) => (
          <div
            key={i}
            className="flex flex-col gap-2 rounded-lg border border-border-default p-3"
          >
            <div className="flex items-center gap-2">
              <LoadingSkeleton className="h-5 w-16" />
              <LoadingSkeleton className="h-3 w-12" />
            </div>
            <LoadingSkeleton className="h-3 w-full" />
            <LoadingSkeleton className="h-3 w-2/3" />
          </div>
        ))}
      </div>
    </div>
  );
}

export default LoadingState;

const paths = {
  case: "M3 6h6l2 2h10v12H3z M3 6V4h6l2 2",
  sources: "M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z",
  file: "M6 3h8l4 4v14H6z M14 3v5h4 M9 12h6 M9 16h6",
  work: "M5 5h4v4H5z M15 15h4v4h-4z M7 9v8h8 M9 7h8v8",
  provider: "M7 3v4 M17 3v4 M5 7h14v4a7 7 0 0 1-14 0z M12 18v4",
  left: "M3 4h18v16H3z M9 4v16",
  right: "M3 4h18v16H3z M15 4v16",
  bottom: "M3 4h18v16H3z M3 14h18",
  close: "M6 6l12 12 M18 6 6 18",
  arrow: "M5 12h14 M14 7l5 5-5 5",
  chevron: "M9 5l7 7-7 7",
  terminal: "M4 6l6 6-6 6 M13 18h7",
  check: "M5 12l4 4L19 6",
  clock: "M12 8v4l3 2 M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0",
  review: "M12 3 2 21h20z M12 9v5 M12 17v1",
  evidence: "M7 3h10v4H7z M7 5H4v16h16V5h-3 M8 11h8 M8 15h5",
  people:
    "M16 21v-3a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v3 M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0 M17 4a4 4 0 0 1 0 8 M19 15a4 4 0 0 1 3 4v2",
} as const;
export type IconName = keyof typeof paths;
export function Icon({ name, size = 18 }: { name: IconName; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[name]} />
    </svg>
  );
}

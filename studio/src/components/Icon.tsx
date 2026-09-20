const paths = {
  case: "M3 6h6l2 2h10v12H3z M3 6V4h6l2 2",
  sources: "M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z",
  file: "M6 3h8l4 4v14H6z M14 3v5h4 M9 12h6 M9 16h6",
  codeFile: "M6 3h8l4 4v14H6z M14 3v5h4 M11 11l-3 3 3 3 M14 11l3 3-3 3",
  pdf: "M6 3h8l4 4v14H6z M14 3v5h4 M8 16v-5h2a2 2 0 0 1 0 4H8 M13 16v-5h2a2 2 0 0 1 2 2v1a2 2 0 0 1-2 2z",
  image: "M5 4h14v16H5z M8 15l3-3 2 2 2-3 3 4 M9 9h.01",
  audio: "M9 18V6l9-2v12 M9 18a3 2 0 1 1-3-2 3 2 0 0 1 3 2 M18 16a3 2 0 1 1-3-2 3 2 0 0 1 3 2",
  video: "M4 6h11v12H4z M15 10l5-3v10l-5-3",
  source: "M8 7h8 M7 4h10v6H7z M7 14h10v6H7z M12 10v4",
  resource: "M8 3v5 M16 3v5 M6 8h12v3a6 6 0 0 1-12 0z M12 17v4",
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
  overview: "M4 5h16v5H4z M4 14h7v5H4z M15 14h5v5h-5z",
  environment: "M4 6h7l2 3h7v10H4z M4 9h16",
  knowledge:
    "M4 5a3 3 0 0 1 3-3h11v17H7a3 3 0 0 0-3 3z M7 2v17 M10 7h5 M10 11h6",
  memory: "M12 3a9 9 0 1 1-8.5 6 M3 4v5h5 M12 7v5l4 2",
  authority: "M12 3l8 3v5c0 5-3.3 8.5-8 10-4.7-1.5-8-5-8-10V6z M9 12l2 2 4-5",
  compute: "M7 3v4 M17 3v4 M5 7h14v4a7 7 0 0 1-14 0z M12 18v3 M8 21h8",
  graph:
    "M6 5a2 2 0 1 0 0 .1 M18 7a2 2 0 1 0 0 .1 M8 18a2 2 0 1 0 0 .1 M8 6l8 1 M7 7l1 9 M17 9l-7 7",
  search: "M11 19a8 8 0 1 1 0-16 8 8 0 0 1 0 16z M17 17l4 4",
  plus: "M12 5v14 M5 12h14",
  trash: "M4 7h16 M9 7V4h6v3 M7 7l1 14h8l1-14 M10 11v6 M14 11v6",
  maximize: "M4 9V4h5 M15 4h5v5 M20 15v5h-5 M9 20H4v-5",
  restore: "M8 4h12v12h-4 M4 8h12v12H4z",
  minimize: "M5 17h14",
  repository: "M4 4h16v16H4z M8 8h8 M8 12h5 M8 16h7",
  database:
    "M5 5c0-2 14-2 14 0v14c0 2-14 2-14 0z M5 5v5c0 2 14 2 14 0V5 M5 10v5c0 2 14 2 14 0",
  arrowBack: "M19 12H5 M10 7l-5 5 5 5",
  back: "M19 12H5 M10 7l-5 5 5 5",
  forward: "M5 12h14 M14 7l5 5-5 5",
  refresh: "M20 6v5h-5 M4 18v-5h5 M18 10a7 7 0 0 0-12-3 M6 14a7 7 0 0 0 12 3",
  warning: "M12 3 2 21h20z M12 9v5 M12 17v1",
  settings: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M4 12H2 M22 12h-2 M12 4V2 M12 22v-2 M6.3 6.3 4.9 4.9 M19.1 19.1l-1.4-1.4 M17.7 6.3l1.4-1.4 M4.9 19.1l1.4-1.4",
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

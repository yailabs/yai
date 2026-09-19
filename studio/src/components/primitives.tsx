import type { ButtonHTMLAttributes, InputHTMLAttributes, ReactNode } from "react";

export function Button({ className = "", ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button className={`ui-button ${className}`} {...props} />;
}
export function IconButton({ className = "", ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button className={`ui-icon-button ${className}`} {...props} />;
}
export function SearchInput(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className="ui-search" type="search" {...props} />;
}
export function Badge({ children, tone = "neutral" }: { children: ReactNode; tone?: "neutral" | "info" | "success" | "warning" | "error" }) {
  return <span className={`ui-badge tone-${tone}`}>{children}</span>;
}
export function EmptyState({ title, body }: { title: string; body: string }) {
  return <div className="ui-empty"><strong>{title}</strong><p>{body}</p></div>;
}
export function PanelHeader({ title, detail, actions }: { title: string; detail?: string; actions?: ReactNode }) {
  return <header className="ui-panel-header"><div><strong>{title}</strong>{detail && <span>{detail}</span>}</div>{actions}</header>;
}

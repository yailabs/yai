import { Badge, Button, EmptyState, IconButton, PanelHeader, SearchInput } from "../components/primitives";
import { Icon } from "../components/Icon";

export function ComponentGallery() {
  return <main className="gallery">
    <header><span>YAI Studio internal</span><h1>UI Foundation</h1><p>Deterministic development surface. This route is excluded from production builds.</p></header>
    <section><h2>Typography</h2><div className="type-specimens"><strong className="type-display">Case Workbench</strong><strong className="type-workspace">Runtime qualification</strong><strong className="type-section">Authority posture</strong><p>Body text explains an application fact without shrinking into a technical caption.</p><code>case:studio-live-qualification</code></div></section>
    <section><h2>Surfaces</h2><div className="surface-specimens"><span className="canvas">Canvas</span><span className="panel">Panel</span><span className="inset">Inset</span><span className="raised">Raised</span><span className="selected">Selected</span></div></section>
    <section><h2>Controls</h2><div className="control-specimens"><Button>Default</Button><Button className="primary">Primary</Button><Button disabled>Disabled</Button><IconButton aria-label="Example icon"><Icon name="settings" /></IconButton><SearchInput placeholder="Search Cases" /><Badge>Local</Badge><Badge tone="success">Available</Badge><Badge tone="warning">Review</Badge><Badge tone="error">Unavailable</Badge></div></section>
    <section><h2>Rows and panels</h2><div className="gallery-panel"><PanelHeader title="Inspector" detail="Selected Case fact" actions={<IconButton aria-label="Close"><Icon name="close" /></IconButton>} /><button className="ui-list-row"><Icon name="case" /><span><strong>case:runtime-qualification</strong><small>Generation 18 · open</small></span><Badge tone="info">Current</Badge></button><button className="ui-list-row selected"><Icon name="review" /><span><strong>Review required</strong><small>Operation is waiting for authority</small></span></button></div></section>
    <section><h2>Empty state</h2><EmptyState title="No provider connected" body="YAI has not bound an inference target to this Case." /></section>
  </main>;
}

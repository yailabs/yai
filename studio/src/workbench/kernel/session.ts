import { NavigationService } from "../../platform/navigation";
import { SurfaceBufferService } from "../surface/buffers";
import { SurfaceGroupService, type SurfaceInput } from "../surface/model";

// Window-local interaction state, partitioned by Case. Switching attachment or
// losing the transport does not dispose drafts, tabs or navigation history.
export class WorkbenchSession {
  private readonly cases = new Map<string, CaseWorkbenchSession>();

  forCase(caseRef: string) {
    let session = this.cases.get(caseRef);
    if (!session) { session = new CaseWorkbenchSession(); this.cases.set(caseRef, session); }
    return session;
  }

  get dirtyCount() { return [...this.cases.values()].reduce((count, session) => count + session.buffers.dirtyCount, 0); }

  dispose() {
    for (const session of this.cases.values()) session.dispose();
    this.cases.clear();
  }
}

class CaseWorkbenchSession {
  readonly surfaces = new SurfaceGroupService();
  readonly buffers = new SurfaceBufferService();
  readonly navigation = new NavigationService();
  readonly archive = new Map<string, SurfaceInput>();

  dispose() { this.surfaces.dispose(); this.buffers.dispose(); this.navigation.dispose(); this.archive.clear(); }
}

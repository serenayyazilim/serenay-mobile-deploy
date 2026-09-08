import { load, type Store } from "@tauri-apps/plugin-store";

const STORE_FILE = "sermobile-settings.json";
const COMPLETED_KEY = "sermobile-onboarding-completed";

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE, { autoSave: true });
  }
  return storeInstance;
}

class OnboardingState {
  completed = $state(true);
  active = $state(false);
  stepIndex = $state(0);

  async init() {
    const store = await getStore();
    const saved = await store.get<boolean>(COMPLETED_KEY);
    this.completed = saved === true;
  }

  start() {
    this.stepIndex = 0;
    this.active = true;
  }

  restart() {
    this.start();
  }

  goTo(index: number) {
    this.stepIndex = index;
  }

  async finish() {
    this.active = false;
    if (!this.completed) {
      this.completed = true;
      const store = await getStore();
      await store.set(COMPLETED_KEY, true);
    }
  }

  skip() {
    this.finish();
  }
}

export const onboardingState = new OnboardingState();

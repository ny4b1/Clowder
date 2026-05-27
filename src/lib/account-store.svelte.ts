import { getAccount, signIn as signInApi, signOutAccount } from "./e621";
import { errMsg } from "./errors";

class AccountStore {
  username = $state<string | null>(null);
  saving = $state(false);
  status = $state("");

  async load() {
    try {
      const result = await getAccount();
      this.username = result.username;
    } catch {
      this.username = null;
    }
  }

  async signIn(username: string, apiKey: string): Promise<boolean> {
    this.saving = true;
    this.status = "verifying";
    try {
      const result = await signInApi(username, apiKey);
      this.username = result.username;
      this.status = "";
      return true;
    } catch (error) {
      this.status = errMsg(error);
      return false;
    } finally {
      this.saving = false;
    }
  }

  async signOut(): Promise<boolean> {
    this.saving = true;
    this.status = "";
    try {
      await signOutAccount();
      this.username = null;
      return true;
    } catch (error) {
      this.status = errMsg(error);
      return false;
    } finally {
      this.saving = false;
    }
  }
}

export const accountStore = new AccountStore();

/**
 * QEMU profile persistence (localStorage)
 */

export interface QemuProfile {
  name: string;
  features: string[];
  bringup?: boolean;
  args?: string[];
}

const STORAGE_KEY = 'qemu_profiles';
const DEFAULT_PROFILE_KEY = 'default_qemu_profile';

/**
 * Load all saved profiles from localStorage
 */
export function loadProfiles(): QemuProfile[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return JSON.parse(stored);
    }
  } catch (err) {
    console.error('Failed to load profiles:', err);
  }
  return [];
}

/**
 * Save a profile to localStorage
 */
export function saveProfile(profile: QemuProfile): void {
  try {
    const profiles = loadProfiles();
    // Replace if exists, otherwise append
    const index = profiles.findIndex((p) => p.name === profile.name);
    if (index >= 0) {
      profiles[index] = profile;
    } else {
      profiles.push(profile);
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(profiles));
  } catch (err) {
    console.error('Failed to save profile:', err);
    throw err;
  }
}

/**
 * Delete a profile from localStorage
 */
export function deleteProfile(name: string): void {
  try {
    const profiles = loadProfiles();
    const filtered = profiles.filter((p) => p.name !== name);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(filtered));
  } catch (err) {
    console.error('Failed to delete profile:', err);
    throw err;
  }
}

/**
 * Get the default profile name
 */
export function getDefaultProfile(): string | null {
  try {
    return localStorage.getItem(DEFAULT_PROFILE_KEY);
  } catch (err) {
    console.error('Failed to get default profile:', err);
    return null;
  }
}

/**
 * Set the default profile name
 */
export function setDefaultProfile(name: string | null): void {
  try {
    if (name) {
      localStorage.setItem(DEFAULT_PROFILE_KEY, name);
    } else {
      localStorage.removeItem(DEFAULT_PROFILE_KEY);
    }
  } catch (err) {
    console.error('Failed to set default profile:', err);
    throw err;
  }
}

import { create } from "zustand";
import type { PageId } from "@/lib/types";

interface AppState {
  currentPage: PageId;
  setCurrentPage: (page: PageId) => void;
}

export const useAppStore = create<AppState>((set) => ({
  currentPage: "home",
  setCurrentPage: (page) => set({ currentPage: page }),
}));

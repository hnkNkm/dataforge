import { create } from 'zustand';

export interface TabItem {
  id: string;
  type: 'query' | 'table';
  title: string;
  content?: string;
  tableName?: string;
  isDirty?: boolean;
}

interface TabStore {
  tabs: TabItem[];
  activeTabId: string | null;

  // Actions
  addTab: (tab: Omit<TabItem, 'id'>) => string;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<TabItem>) => void;
  openTableTab: (tableName: string) => void;
  openQueryTab: (title?: string) => void;
}

export const useTabStore = create<TabStore>((set, get) => ({
  tabs: [{
    id: 'default-query',
    type: 'query',
    title: '新規クエリ',
    content: 'SELECT * FROM '
  }],
  activeTabId: 'default-query',

  addTab: (tab) => {
    const id = `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const newTab = { ...tab, id };
    set(state => ({
      tabs: [...state.tabs, newTab],
      activeTabId: id
    }));
    return id;
  },

  removeTab: (id) => {
    set(state => {
      const tabs = state.tabs.filter(t => t.id !== id);
      // 最低1つのタブは残す
      if (tabs.length === 0) {
        tabs.push({
          id: 'default-query',
          type: 'query',
          title: '新規クエリ',
          content: 'SELECT * FROM '
        });
      }
      // アクティブタブが削除される場合は最後のタブをアクティブに
      const activeTabId = state.activeTabId === id
        ? tabs[tabs.length - 1].id
        : state.activeTabId;

      return { tabs, activeTabId };
    });
  },

  setActiveTab: (id) => {
    set({ activeTabId: id });
  },

  updateTab: (id, updates) => {
    set(state => ({
      tabs: state.tabs.map(tab =>
        tab.id === id ? { ...tab, ...updates } : tab
      )
    }));
  },

  openTableTab: (tableName) => {
    const existingTab = get().tabs.find(t => t.type === 'table' && t.tableName === tableName);
    if (existingTab) {
      set({ activeTabId: existingTab.id });
    } else {
      get().addTab({
        type: 'table',
        title: tableName,
        tableName: tableName
      });
    }
  },

  openQueryTab: (title = '新規クエリ') => {
    get().addTab({
      type: 'query',
      title: title,
      content: 'SELECT * FROM '
    });
  }
}));
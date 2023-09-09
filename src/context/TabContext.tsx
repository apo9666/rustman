import React, { createContext, useReducer, type ReactNode, type Dispatch } from 'react'

export interface Response {
  url: string
  status: number
  ok: boolean
  headers: Record<string, string>
  rawHeaders: Record<string, string[]>
  data: string
}

export enum MethodEnum {
  GET = 'get',
  POST = 'post',
  PUT = 'put',
  PATCH = 'patch',
  DELETE = 'delete',
  OPTIONS = 'options',
  HEAD = 'head',
  TRACE = 'trace',
}

export interface TabContent {
  method: MethodEnum
  url: string
  body: string
  response?: Response
}

export interface Tab {
  id: number
  label: string
  content: TabContent
}

interface TabState {
  activeTabId: number
  lastTabId: number
  tabs: Tab[]
}

const tabInitialState: Tab = {
  id: 0,
  label: 'New Tab',
  content: {
    url: '',
    method: MethodEnum.GET,
    body: ''
  }
}

const tabsInitialState: TabState = {
  activeTabId: 0,
  lastTabId: 0,
  tabs: [tabInitialState]
}

type Action =
  | { type: 'SET_TABS', payload: Tab[] }
  | { type: 'ADD_TAB', payload?: Omit<Tab, 'id'> }
  | { type: 'REMOVE_TAB', payload: Tab }
  | { type: 'SET_ACTIVE_TAB', payload: number }
  | { type: 'UPDATE_CONTENT', payload: { id: number, content: Partial<TabContent> } }

export const TabContext = createContext<{ state: TabState, dispatch: Dispatch<Action> }>({
  state: tabsInitialState,
  dispatch: () => { /* empty */ }
})

const tabReducer = (state: TabState, action: Action): TabState => {
  switch (action.type) {
    case 'SET_TABS':
      return { ...state, tabs: action.payload }
    case 'ADD_TAB': {
      console.log(action.payload)
      const newId = state.lastTabId + 1
      return {
        ...state,
        lastTabId: newId,
        activeTabId: newId,
        tabs: [...state.tabs, {
          ...(action.payload ?? tabInitialState),
          id: newId
        }]
      }
    }
    case 'REMOVE_TAB': {
      const newTabs = state.tabs.filter(tab => tab.id !== action.payload.id)
      return {
        ...state,
        tabs: newTabs,
        activeTabId: action.payload.id === state.activeTabId
          ? (newTabs[newTabs.length - 1]?.id ?? -1)
          : state.activeTabId
      }
    }
    case 'SET_ACTIVE_TAB':
      return { ...state, activeTabId: action.payload }
    case 'UPDATE_CONTENT': {
      const mapTab = (tab: Tab): Tab => {
        if (tab.id === action.payload.id) {
          return {
            ...tab,
            content: {
              ...tab.content,
              ...action.payload.content
            }
          }
        }
        return tab
      }

      return {
        ...state,
        tabs: state.tabs.map(mapTab)
      }
    }
  }
}

export const TabProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(tabReducer, tabsInitialState)
  return <TabContext.Provider value={{ state, dispatch }}>{children}</TabContext.Provider>
}

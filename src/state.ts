import { hookstate } from '@hookstate/core'
import { type OpenAPIObject } from 'openapi3-ts/oas31'

export interface TreeNode {
  id: number
  label: string
  content?: TabContent
  expanded: boolean
  children?: TreeNode[]
}

interface TreeState {
  tree: TreeNode
  lastTreeId: number
}

const init: OpenAPIObject = {
  info: {
    title: 'test',
    version: '1'
  },
  openapi: ''
}

const treeInitialState: TreeState = {
  tree: {
    id: 0,
    label: 'Root',
    expanded: true,
    children: [
    ]
  },
  lastTreeId: 0
}

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

export interface Header {
  id: number
  enable: boolean
  key: string
  value: string
}

export const headerInitialState: Header = {
  id: 0,
  enable: true,
  key: '',
  value: ''
}

export interface TabContent {
  method: MethodEnum
  url: string
  body: string
  headers: Header[]
  response: Response
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

export const tabInitialState: Tab = {
  id: 0,
  label: 'New Tab',
  content: {
    url: '',
    headers: [headerInitialState],
    method: MethodEnum.GET,
    body: '',
    response: {
      data: '',
      headers: {},
      ok: true,
      rawHeaders: {},
      status: 200,
      url: ''
    }
  }
}

const tabsInitialState: TabState = {
  activeTabId: 0,
  lastTabId: 0,
  tabs: [tabInitialState]
}

export const treeState = hookstate(treeInitialState)
export const tabState = hookstate(tabsInitialState)

export const apiState = hookstate<OpenAPIObject>(init)

export type PaneDefinition = {
  id: string,
  component: ComponentAppType,
  height: number,
}

export type ComponentAppType = 'ChatFront' | 'TabSync' | 'Unknown';
// Cross-reference and game content type definitions

export type ReferenceType = 'spell' | 'item' | 'creature' | 'condition' | 'race' | 'class' | 'classFeature' | 'subclass' | 'subclassFeature' | 'background' | 'feat' | 'action' | 'language' | 'trap' | 'hazard'

export interface ReferenceData {
  success: boolean
  data?: any
  message?: string
}

export interface SpellData {
  name: string
  source?: string
  level?: number
  school?: string
  range?: any
  components?: any
  duration?: any
  entries?: string[]
  damageInflict?: string[]
  savingThrow?: string[]
  conditionInflict?: string[]
  entriesHigherLevel?: any[]
  classes?: {
    fromClassList?: Array<{ name: string; source?: string }>
  }
}

export interface ItemData {
  name: string
  source?: string
  type?: string
  rarity?: string
  weight?: number
  value?: number
  entries?: string[]
  ac?: number
  dmg1?: string
  dmgType?: string
  property?: string[]
  range?: string
  reload?: number
  bonusWeapon?: string
  bonusAc?: string
  resist?: string[]
  attachedSpells?: string[]
}

export interface CreatureData {
  name: string
  source?: string
  size?: string[]
  type?: string | { type: string; tags?: string[] }
  alignment?: any[]
  ac?: any[]
  hp?: any
  speed?: any
  str?: number
  dex?: number
  con?: number
  int?: number
  wis?: number
  cha?: number
  save?: any
  skill?: any
  senses?: string[]
  languages?: string[]
  cr?: string | number | any
  immune?: string[]
  resist?: string[]
  vulnerable?: string[]
  conditionImmune?: string[]
  trait?: any[]
  action?: any[]
  reaction?: any[]
  legendary?: any[]
  legendaryGroup?: any
  environment?: string[]
}

export interface ConditionData {
  name: string
  source?: string
  entries?: string[]
}

export interface RaceData {
  name: string
  source?: string
  ability?: any[]
  size?: string[]
  speed?: any
  entries?: any[]
  darkvision?: number
  traitTags?: string[]
  languageProficiencies?: any[]
  skillProficiencies?: any[]
  age?: any
  heightAndWeight?: any
}

export interface ClassData {
  name: string
  source?: string
  hd?: { number: number; faces: number }
  proficiency?: string[]
  classTableGroups?: any[]
  startingProficiencies?: any
  startingEquipment?: any
  classFeatures?: any[]
  subclassTitle?: string
  subclasses?: any[]
}

export interface BackgroundData {
  name: string
  source?: string
  entries?: any[]
  skillProficiencies?: any[]
  languageProficiencies?: any[]
  toolProficiencies?: any[]
  startingEquipment?: any[]
  feature?: any[]
  characteristics?: any[]
}

export interface FeatData {
  name: string
  source?: string
  entries?: any[]
  prerequisite?: any[]
  ability?: any[]
}

export interface ClassFeatureData {
  name: string
  source?: string
  className?: string
  classSource?: string
  level?: number
  entries?: any[]
}

export interface SubclassData {
  name: string
  source?: string
  className?: string
  classSource?: string
  shortName?: string
  subclassFeatures?: string[]
  entries?: any[]
}

export interface SubclassFeatureData {
  name: string
  source?: string
  className?: string
  classSource?: string
  subclassShortName?: string
  subclassSource?: string
  level?: number
  entries?: any[]
}
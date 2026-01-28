<template>
  <MainLayout>
    <div class="character-sheet">
      <div v-if="loading" class="loading-state">Loading character...</div>

      <div v-else-if="error" class="error-state">
        <p>{{ error }}</p>
        <button @click="loadCharacter" class="btn btn-primary">Retry</button>
      </div>

      <template v-else-if="character">
        <!-- Header -->
        <div class="sheet-header">
          <div class="header-content">
            <button @click="goBack" class="btn-back">Back</button>
            <h1 class="character-name">{{ character.name }}</h1>
            <div class="character-subtitle">
              Level {{ totalLevel }} {{ character.race_name || '' }} {{ classString }}
            </div>
            <div v-if="character.background_name" class="character-background">
              {{ character.background_name }}
            </div>
          </div>
          <div class="header-actions">
            <span v-if="character.is_npc === 1" class="npc-badge">NPC</span>
            <span v-else-if="character.player_name" class="player-name">
              Player: {{ character.player_name }}
            </span>
            <button @click="printCharacter" class="btn btn-secondary">Print PDF</button>
          </div>
        </div>

        <!-- Tab Navigation -->
        <div class="tab-navigation">
          <button
            @click="activeTab = 'character'"
            :class="['tab-button', { active: activeTab === 'character' }]"
          >
            Character
          </button>
          <button
            @click="activeTab = 'equipment'"
            :class="['tab-button', { active: activeTab === 'equipment' }]"
          >
            Equipment
          </button>
          <button
            v-if="characterIsSpellcaster"
            @click="activeTab = 'spells'"
            :class="['tab-button', { active: activeTab === 'spells' }]"
          >
            Spells
          </button>
          <button
            @click="activeTab = 'details'"
            :class="['tab-button', { active: activeTab === 'details' }]"
          >
            Details
          </button>
        </div>

        <!-- Character Tab -->
        <div v-if="activeTab === 'character'" class="sheet-content three-columns">
          <!-- Left Column: Abilities & Combat -->
          <div class="sheet-column">
            <!-- Ability Scores -->
            <section class="sheet-section">
              <h2>Ability Scores</h2>
              <div class="ability-grid">
                <div v-for="ability in ABILITIES" :key="ability" class="ability-box">
                  <div class="ability-name">{{ ability.slice(0, 3).toUpperCase() }}</div>
                  <div class="ability-value">{{ character[ability] }}</div>
                  <div class="ability-modifier">
                    {{ formatMod(getModifier(character[ability])) }}
                  </div>
                </div>
              </div>
            </section>

            <!-- Combat Stats -->
            <section class="sheet-section">
              <h2>Combat</h2>
              <div class="combat-grid">
                <div class="combat-stat">
                  <span class="stat-label">Armor Class</span>
                  <span class="stat-value">{{ baseAC }}</span>
                  <span v-if="equippedArmor" class="stat-note">{{ equippedArmor.item_name }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Initiative</span>
                  <span class="stat-value">{{ formatMod(getModifier(character.dexterity)) }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Speed</span>
                  <span class="stat-value">{{ speed }} ft</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Passive Perception</span>
                  <span class="stat-value">{{ passivePerception }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Hit Dice</span>
                  <span class="stat-value">{{ hitDice }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Proficiency</span>
                  <span class="stat-value">{{ formatMod(profBonus) }}</span>
                </div>
              </div>
            </section>

            <!-- Saving Throws -->
            <section class="sheet-section">
              <h2>Saving Throws</h2>
              <div class="saves-list">
                <div v-for="ability in ABILITIES" :key="ability" class="save-item">
                  <span class="save-proficient" :class="{ active: isSaveProficient(ability) }">
                    *
                  </span>
                  <span class="save-name">{{ ability }}</span>
                  <span class="save-bonus">
                    {{ formatMod(getSaveBonus(character, ability, character[ability])) }}
                  </span>
                </div>
              </div>
            </section>

            <!-- Attacks -->
            <section v-if="attacks.length > 0" class="sheet-section">
              <h2>Attacks</h2>
              <div class="attacks-list">
                <div v-for="attack in attacks" :key="attack.name" class="attack-item">
                  <span class="attack-name">{{ attack.name }}</span>
                  <span class="attack-bonus">{{ formatMod(attack.attackBonus) }}</span>
                  <span class="attack-damage">{{ attack.damage }}</span>
                </div>
              </div>
            </section>
          </div>

          <!-- Middle Column: Skills -->
          <div class="sheet-column">
            <section class="sheet-section skills-section">
              <h2>Skills</h2>
              <div class="skills-list">
                <div v-for="skill in ALL_SKILLS" :key="skill.name" class="skill-item">
                  <span
                    class="skill-proficient"
                    :class="{
                      active: isSkillProficient(skill.name),
                      expertise: hasExpertise(skill.name),
                    }"
                  >
                    {{ hasExpertise(skill.name) ? '**' : '*' }}
                  </span>
                  <span class="skill-name">{{ skill.name }}</span>
                  <span class="skill-ability">({{ skill.ability.slice(0, 3) }})</span>
                  <span class="skill-bonus">
                    {{ formatMod(getSkillBonus(character, skill.name, character[skill.ability])) }}
                  </span>
                </div>
              </div>
            </section>
          </div>

          <!-- Right Column: Features & Proficiencies -->
          <div class="sheet-column">
            <!-- Proficiencies -->
            <section class="sheet-section">
              <h2>Proficiencies</h2>
              <div v-if="armorProficiencies.length" class="proficiency-group">
                <strong>Armor:</strong> {{ armorProficiencies.map((p) => p.name).join(', ') }}
              </div>
              <div v-if="weaponProficiencies.length" class="proficiency-group">
                <strong>Weapons:</strong> {{ weaponProficiencies.map((p) => p.name).join(', ') }}
              </div>
              <div v-if="toolProficiencies.length" class="proficiency-group">
                <strong>Tools:</strong> {{ toolProficiencies.map((p) => p.name).join(', ') }}
              </div>
              <div v-if="languages.length" class="proficiency-group">
                <strong>Languages:</strong> {{ languages.map((p) => p.name).join(', ') }}
              </div>
              <div v-if="!hasProficiencies" class="empty-proficiencies">
                No proficiencies recorded
              </div>
            </section>

            <!-- Class Features -->
            <section v-if="classFeatures.length > 0" class="sheet-section">
              <h2>Class Features</h2>
              <div class="features-list">
                <div
                  v-for="feature in classFeatures"
                  :key="`${feature.class_name}-${feature.name}-${feature.level}`"
                  class="feature-item"
                  :class="{ expanded: isFeatureExpanded(feature) }"
                >
                  <div class="feature-header" @click="toggleFeatureExpansion(feature)">
                    <span class="feature-name">
                      <span class="expand-icon">{{ isFeatureExpanded(feature) ? '▼' : '▶' }}</span>
                      {{ feature.name }}
                    </span>
                    <span class="feature-meta">
                      <span v-if="feature.subclass_name" class="subclass-badge">{{ feature.subclass_name }}</span>
                      {{ feature.class_name }} {{ feature.level }}
                    </span>
                  </div>
                  <div v-if="isFeatureExpanded(feature)" class="feature-details">
                    <div v-if="isFeatureLoading(feature)" class="feature-loading">Loading...</div>
                    <div v-else-if="getFeatureDescription(feature)" class="feature-description" v-html="getFeatureDescription(feature)"></div>
                    <div v-else class="feature-no-desc">No description available</div>
                  </div>
                </div>
              </div>
            </section>

            <!-- Spellcasting Summary -->
            <section v-if="characterIsSpellcaster" class="sheet-section">
              <h2>Spellcasting</h2>
              <div class="spell-stats">
                <div class="spell-stat">
                  <span class="stat-label">Spell Save DC</span>
                  <span class="stat-value">{{ spellSaveDC }}</span>
                </div>
                <div class="spell-stat">
                  <span class="stat-label">Spell Attack</span>
                  <span class="stat-value">{{ formatMod(spellAttackBonus || 0) }}</span>
                </div>
                <div class="spell-stat">
                  <span class="stat-label">Ability</span>
                  <span class="stat-value spell-ability">
                    {{ spellcastingAbility?.toUpperCase().slice(0, 3) }}
                  </span>
                </div>
              </div>
              <p class="spell-note">See Spells tab for full spell list</p>
            </section>

            <!-- Personality -->
            <section v-if="hasPersonality" class="sheet-section">
              <h2>Personality</h2>
              <div v-if="character.traits" class="personality-item">
                <strong>Traits:</strong> {{ character.traits }}
              </div>
              <div v-if="character.ideals" class="personality-item">
                <strong>Ideals:</strong> {{ character.ideals }}
              </div>
              <div v-if="character.bonds" class="personality-item">
                <strong>Bonds:</strong> {{ character.bonds }}
              </div>
              <div v-if="character.flaws" class="personality-item">
                <strong>Flaws:</strong> {{ character.flaws }}
              </div>
            </section>
          </div>
        </div>

        <!-- Equipment Tab -->
        <div v-else-if="activeTab === 'equipment'" class="sheet-content single-column">
          <!-- Currency -->
          <section class="sheet-section">
            <h2>Currency</h2>
            <div class="currency-display">
              <div class="currency-item large">
                <span class="currency-icon pp">PP</span>
                <span class="currency-value">{{ character.pp }}</span>
              </div>
              <div class="currency-item large">
                <span class="currency-icon gp">GP</span>
                <span class="currency-value">{{ character.gp }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-icon ep">EP</span>
                <span class="currency-value">{{ character.ep }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-icon sp">SP</span>
                <span class="currency-value">{{ character.sp }}</span>
              </div>
              <div class="currency-item">
                <span class="currency-icon cp">CP</span>
                <span class="currency-value">{{ character.cp }}</span>
              </div>
            </div>
          </section>

          <!-- Equipped Items -->
          <section class="sheet-section">
            <h2>Equipped Items</h2>
            <div v-if="equippedItems.length === 0" class="empty-state">
              No items equipped
            </div>
            <div v-else class="item-cards">
              <div
                v-for="item in equippedItems"
                :key="item.id"
                class="item-card"
                :class="{ expanded: isItemExpanded(item.item_name, item.item_source) }"
              >
                <div
                  class="item-card-header"
                  @click="toggleItemDetails(item.item_name, item.item_source)"
                >
                  <span class="item-name">{{ item.item_name }}</span>
                  <span class="item-meta">
                    <span v-if="item.attuned" class="item-attuned">Attuned</span>
                    <span class="item-source">{{ item.item_source }}</span>
                    <span class="expand-icon">{{ isItemExpanded(item.item_name, item.item_source) ? '−' : '+' }}</span>
                  </span>
                </div>
                <div
                  v-if="isItemExpanded(item.item_name, item.item_source)"
                  class="item-card-details"
                >
                  <template v-if="getItemDetail(item.item_name, item.item_source)">
                    <div class="item-detail-row" v-if="getItemDetail(item.item_name, item.item_source)?.rarity">
                      <span class="detail-label">Rarity:</span>
                      <span class="detail-value rarity" :class="getItemDetail(item.item_name, item.item_source)?.rarity?.toLowerCase()">
                        {{ getItemDetail(item.item_name, item.item_source)?.rarity }}
                      </span>
                    </div>
                    <div class="item-detail-row" v-if="getItemProperties(getItemDetail(item.item_name, item.item_source)!).length > 0">
                      <span class="detail-label">Properties:</span>
                      <span class="detail-value">{{ getItemProperties(getItemDetail(item.item_name, item.item_source)!).join(', ') }}</span>
                    </div>
                    <div class="item-description" v-if="getItemDescription(getItemDetail(item.item_name, item.item_source)!)">
                      {{ getItemDescription(getItemDetail(item.item_name, item.item_source)!) }}
                    </div>
                  </template>
                  <div v-else class="loading-details">Loading details...</div>
                </div>
              </div>
            </div>
          </section>

          <!-- Full Inventory -->
          <section class="sheet-section">
            <div class="section-header-row">
              <h2>Inventory</h2>
              <button @click="openInventory" class="btn btn-secondary btn-sm">Manage</button>
            </div>
            <div v-if="loadingInventory" class="loading-inventory">Loading inventory...</div>
            <div v-else-if="inventory.length === 0" class="empty-state">
              No items in inventory
            </div>
            <div v-else class="item-cards">
              <div
                v-for="item in inventory"
                :key="item.id"
                class="item-card"
                :class="{ expanded: isItemExpanded(item.item_name, item.item_source) }"
              >
                <div
                  class="item-card-header"
                  @click="toggleItemDetails(item.item_name, item.item_source)"
                >
                  <span class="item-name">
                    {{ item.item_name }}
                    <span v-if="item.quantity > 1" class="item-qty">x{{ item.quantity }}</span>
                  </span>
                  <span class="item-meta">
                    <span v-if="item.equipped" class="item-equipped-badge">Equipped</span>
                    <span v-if="item.attuned" class="item-attuned">Attuned</span>
                    <span class="expand-icon">{{ isItemExpanded(item.item_name, item.item_source) ? '−' : '+' }}</span>
                  </span>
                </div>
                <div
                  v-if="isItemExpanded(item.item_name, item.item_source)"
                  class="item-card-details"
                >
                  <template v-if="getItemDetail(item.item_name, item.item_source)">
                    <div class="item-detail-row" v-if="getItemDetail(item.item_name, item.item_source)?.rarity">
                      <span class="detail-label">Rarity:</span>
                      <span class="detail-value rarity" :class="getItemDetail(item.item_name, item.item_source)?.rarity?.toLowerCase()">
                        {{ getItemDetail(item.item_name, item.item_source)?.rarity }}
                      </span>
                    </div>
                    <div class="item-detail-row" v-if="getItemProperties(getItemDetail(item.item_name, item.item_source)!).length > 0">
                      <span class="detail-label">Properties:</span>
                      <span class="detail-value">{{ getItemProperties(getItemDetail(item.item_name, item.item_source)!).join(', ') }}</span>
                    </div>
                    <div class="item-description" v-if="getItemDescription(getItemDetail(item.item_name, item.item_source)!)">
                      {{ getItemDescription(getItemDetail(item.item_name, item.item_source)!) }}
                    </div>
                    <div v-if="item.notes" class="item-notes">
                      <span class="detail-label">Notes:</span> {{ item.notes }}
                    </div>
                  </template>
                  <div v-else class="loading-details">Loading details...</div>
                </div>
              </div>
            </div>
          </section>
        </div>

        <!-- Spells Tab -->
        <div v-else-if="activeTab === 'spells'" class="sheet-content single-column">
          <!-- Spellcasting Stats -->
          <section class="sheet-section">
            <h2>Spellcasting</h2>
            <!-- Multiclass: show stats for each spellcasting class -->
            <template v-if="isMulticlassSpellcaster">
              <div
                v-for="stats in allSpellcastingStats"
                :key="stats.className"
                class="spell-stats-row multiclass"
              >
                <div class="spell-class-label">{{ stats.className }}</div>
                <div class="spell-stat-box">
                  <span class="stat-label">Spell Save DC</span>
                  <span class="stat-value large">{{ stats.saveDC }}</span>
                </div>
                <div class="spell-stat-box">
                  <span class="stat-label">Spell Attack</span>
                  <span class="stat-value large">{{ formatMod(stats.attackBonus) }}</span>
                </div>
                <div class="spell-stat-box">
                  <span class="stat-label">Ability</span>
                  <span class="stat-value large">{{ stats.abilityAbbrev }}</span>
                </div>
              </div>
            </template>
            <!-- Single class: show simple row -->
            <div v-else class="spell-stats-row">
              <div class="spell-stat-box">
                <span class="stat-label">Spell Save DC</span>
                <span class="stat-value large">{{ spellSaveDC }}</span>
              </div>
              <div class="spell-stat-box">
                <span class="stat-label">Spell Attack</span>
                <span class="stat-value large">{{ formatMod(spellAttackBonus || 0) }}</span>
              </div>
              <div class="spell-stat-box">
                <span class="stat-label">Spellcasting Ability</span>
                <span class="stat-value large">{{ spellcastingAbility?.toUpperCase().slice(0, 3) }}</span>
              </div>
            </div>
          </section>

          <!-- Spell Slots -->
          <section v-if="spellSlots" class="sheet-section">
            <h2>Spell Slots</h2>
            <p class="spell-info-note">
              Maximum slots shown below. Track used slots on paper.
            </p>
            <div class="spell-slots-grid">
              <div class="spell-slot-row cantrip-row">
                <span class="slot-level">Cantrips</span>
                <span class="slot-unlimited">Unlimited</span>
              </div>
              <div v-for="level in 9" :key="level" class="spell-slot-row">
                <template v-if="spellSlots[level]">
                  <span class="slot-level">Level {{ level }}</span>
                  <div class="slot-boxes">
                    <span
                      v-for="n in spellSlots[level]"
                      :key="n"
                      class="slot-box"
                    ></span>
                  </div>
                  <span class="slot-count">{{ spellSlots[level] }} slots</span>
                </template>
              </div>
            </div>
          </section>

          <!-- Available Spells -->
          <section class="sheet-section">
            <h2>Available Spells</h2>
            <p class="spell-info-note">
              These spells are available to your class. Track prepared/known spells on paper.
            </p>

            <div v-if="loadingSpells" class="loading-state">Loading spells...</div>

            <div v-else-if="classSpells.length === 0" class="empty-state">
              No spells available for your class configuration.
            </div>

            <div v-else class="spell-list-container">
              <div v-for="level in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]" :key="level">
                <div v-if="spellsByLevel[level]?.length" class="spell-level-group">
                  <h3
                    class="spell-level-header collapsible"
                    :class="{ collapsed: isSpellLevelCollapsed(level) }"
                    @click="toggleSpellLevel(level)"
                  >
                    <span class="collapse-icon">{{ isSpellLevelCollapsed(level) ? '▶' : '▼' }}</span>
                    {{ getLevelDisplay(level) }}
                    <span class="spell-count">({{ spellsByLevel[level].length }})</span>
                  </h3>
                  <div v-show="!isSpellLevelCollapsed(level)" class="spell-cards">
                    <div
                      v-for="spell in spellsByLevel[level]"
                      :key="`${spell.name}|${spell.source}`"
                      class="spell-card"
                      :class="{ expanded: isSpellExpanded(spell.name, spell.source) }"
                    >
                      <div
                        class="spell-card-header"
                        @click="toggleSpellDetails(spell.name, spell.source)"
                      >
                        <span class="spell-name">
                          {{ spell.name }}
                          <span v-if="spell.ritual" class="spell-tag ritual">R</span>
                          <span v-if="spell.concentration" class="spell-tag conc">C</span>
                        </span>
                        <span class="spell-meta">
                          <span class="spell-school">{{ getSchoolName(spell.school) }}</span>
                          <span class="expand-icon">{{ isSpellExpanded(spell.name, spell.source) ? '−' : '+' }}</span>
                        </span>
                      </div>
                      <div
                        v-if="isSpellExpanded(spell.name, spell.source)"
                        class="spell-card-details"
                      >
                        <div class="spell-stats-mini">
                          <div class="spell-stat-mini">
                            <span class="label">Casting Time:</span>
                            <span>{{ getSpellCastingTime(spell) }}</span>
                          </div>
                          <div class="spell-stat-mini">
                            <span class="label">Range:</span>
                            <span>{{ getSpellRange(spell) }}</span>
                          </div>
                          <div class="spell-stat-mini">
                            <span class="label">Components:</span>
                            <span>{{ getSpellComponents(spell) }}</span>
                          </div>
                          <div class="spell-stat-mini">
                            <span class="label">Duration:</span>
                            <span>{{ getSpellDuration(spell) }}</span>
                          </div>
                        </div>
                        <div class="spell-description">
                          {{ getSpellDescription(spell) }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </div>

        <!-- Details Tab -->
        <div v-else-if="activeTab === 'details'" class="sheet-content single-column">
          <!-- Background -->
          <section v-if="character.background_name" class="sheet-section">
            <h2>Background: {{ character.background_name }}</h2>

            <template v-if="backgroundDetails">
              <!-- Background Proficiencies -->
              <div class="details-card">
                <h3>Proficiencies</h3>
                <div class="proficiency-grid">
                  <div v-if="getBackgroundSkillProficiencies().length > 0" class="prof-item">
                    <span class="prof-label">Skills:</span>
                    <span>{{ getBackgroundSkillProficiencies().join(', ') }}</span>
                  </div>
                  <div v-if="getBackgroundToolProficiencies().length > 0" class="prof-item">
                    <span class="prof-label">Tools:</span>
                    <span>{{ getBackgroundToolProficiencies().join(', ') }}</span>
                  </div>
                  <div v-if="getBackgroundLanguages()" class="prof-item">
                    <span class="prof-label">Languages:</span>
                    <span>{{ getBackgroundLanguages() }}</span>
                  </div>
                </div>
              </div>

              <!-- Background Equipment -->
              <div v-if="getBackgroundEquipment()" class="details-card">
                <h3>Starting Equipment</h3>
                <p>{{ getBackgroundEquipment() }}</p>
              </div>

              <!-- Background Feature -->
              <div v-if="getBackgroundFeature()" class="details-card feature-card">
                <h3>Feature: {{ getBackgroundFeature()?.name }}</h3>
                <p>{{ getBackgroundFeature()?.description }}</p>
              </div>
            </template>

            <p v-else class="loading-text">Loading background details...</p>
          </section>

          <!-- NPC Info -->
          <section v-if="character.is_npc === 1 && hasNpcInfo" class="sheet-section">
            <h2>NPC Details</h2>
            <div class="npc-details-grid">
              <div v-if="character.role" class="npc-detail-item">
                <span class="npc-label">Role</span>
                <span class="npc-value">{{ character.role }}</span>
              </div>
              <div v-if="character.location" class="npc-detail-item">
                <span class="npc-label">Location</span>
                <span class="npc-value">{{ character.location }}</span>
              </div>
              <div v-if="character.faction" class="npc-detail-item">
                <span class="npc-label">Faction</span>
                <span class="npc-value">{{ character.faction }}</span>
              </div>
            </div>
          </section>

          <!-- Classes -->
          <section class="sheet-section">
            <h2>Classes</h2>
            <div v-if="character.classes.length === 0" class="empty-state">No classes</div>
            <div v-else class="class-details-list">
              <div v-for="cls in character.classes" :key="cls.id" class="class-detail-card">
                <div class="class-header">
                  <h3>
                    {{ cls.class_name }}
                    <span class="class-level-badge">Level {{ cls.level }}</span>
                  </h3>
                </div>

                <!-- Class Mechanical Info -->
                <div class="class-stats-grid">
                  <div class="class-stat">
                    <span class="class-stat-label">Hit Die</span>
                    <span class="class-stat-value">{{ getClassHitDice(cls.class_name) || '—' }}</span>
                  </div>
                  <div class="class-stat">
                    <span class="class-stat-label">Primary Ability</span>
                    <span class="class-stat-value">{{ getClassPrimaryAbility(cls.class_name) || '—' }}</span>
                  </div>
                  <div v-if="getClassSpellcastingAbility(cls.class_name)" class="class-stat">
                    <span class="class-stat-label">Spellcasting</span>
                    <span class="class-stat-value spellcaster">{{ getClassSpellcastingAbility(cls.class_name) }}</span>
                  </div>
                  <div class="class-stat">
                    <span class="class-stat-label">Saving Throws</span>
                    <span class="class-stat-value">{{ getClassSavingThrows(cls.class_name).join(', ') || '—' }}</span>
                  </div>
                </div>

                <!-- Starting Proficiencies -->
                <div class="class-proficiencies">
                  <h4>Starting Proficiencies</h4>
                  <div class="proficiency-grid">
                    <div v-if="getClassProficiencies(cls.class_name).armor" class="prof-item">
                      <span class="prof-label">Armor:</span>
                      <span v-html="getClassProficiencies(cls.class_name).armor"></span>
                    </div>
                    <div v-if="getClassProficiencies(cls.class_name).weapons" class="prof-item">
                      <span class="prof-label">Weapons:</span>
                      <span v-html="getClassProficiencies(cls.class_name).weapons"></span>
                    </div>
                    <div v-if="getClassProficiencies(cls.class_name).tools" class="prof-item">
                      <span class="prof-label">Tools:</span>
                      <span>{{ getClassProficiencies(cls.class_name).tools }}</span>
                    </div>
                    <div v-if="getClassProficiencies(cls.class_name).skills" class="prof-item">
                      <span class="prof-label">Skills:</span>
                      <span>{{ getClassProficiencies(cls.class_name).skills }}</span>
                    </div>
                  </div>
                </div>

                <!-- Subclass -->
                <div v-if="cls.subclass_name" class="subclass-section">
                  <h4 class="subclass-header">
                    Subclass: {{ cls.subclass_name }}
                  </h4>
                  <div v-if="getSubclassDescription(cls.class_name, cls.subclass_name)" class="subclass-description">
                    {{ getSubclassDescription(cls.class_name, cls.subclass_name) }}
                  </div>
                </div>

                <!-- Class Features for this class -->
                <div class="class-features-section">
                  <h4>Features</h4>
                  <div class="features-by-level">
                    <template v-for="level in getFeatureLevels(cls.class_name, cls.subclass_name)" :key="level">
                      <div class="feature-level-group">
                        <span class="level-label">{{ formatOrdinal(level) }} Level:</span>
                        <span class="feature-links">
                          <a
                            v-for="(feature, idx) in getFeaturesAtLevel(cls.class_name, cls.subclass_name, level)"
                            :key="feature.name + (feature.subclass_name || '')"
                            href="#"
                            :class="['cross-ref-link', 'feature-ref', { 'subclass-feature': feature.subclass_name }]"
                            @click.prevent="openFeatureModal(feature)"
                          >{{ feature.name }}{{ idx < getFeaturesAtLevel(cls.class_name, cls.subclass_name, level).length - 1 ? ', ' : '' }}</a>
                        </span>
                      </div>
                    </template>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </div>
      </template>
    </div>

    <!-- Inventory Manager Dialog -->
    <InventoryManager
      v-if="character"
      :visible="showInventory"
      :character-id="characterId"
      :character-data="character"
      @close="showInventory = false"
      @updated="loadInventory"
    />

    <!-- Print Dialog -->
    <CharacterPrintDialog
      v-if="character"
      :visible="showPrintDialog"
      :character-id="characterId"
      :character-name="character.name"
      @close="showPrintDialog = false"
    />

    <!-- Cross-Reference Modal -->
    <AppModal
      :visible="modalContent.visible"
      :title="modalContent.title"
      size="md"
      @close="closeModal"
    >
      <div class="dnd-content" v-html="modalContent.content"></div>
    </AppModal>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import InventoryManager from '../components/InventoryManager.vue'
import { CharacterPrintDialog } from '../../../components/print'
import AppModal from '@/components/shared/AppModal.vue'
import { useCharacterStore } from '../../../stores/characters'
import { useCrossReferences } from '../../sources/composables/useCrossReferences'
import { renderModalContent } from '../../sources/formatters/modalFormatters'
import type { Character, CharacterInventory } from '../../../types/character'

// Class feature from catalog
interface ClassFeature {
  name: string
  source: string
  class_name: string
  class_source: string
  level: number
  data: string
  // For subclass features
  subclass_name?: string
  subclass_short_name?: string
  subclass_source?: string
}

// Item detail from catalog
interface ItemDetail {
  name: string
  source: string
  item_type: string | null
  rarity: string | null
  data: Record<string, unknown>
  fluff: string | null
}

// Spell info from catalog
interface SpellInfo {
  name: string
  source: string
  level: number
  school: string | null
  ritual: boolean
  concentration: boolean
  data: Record<string, unknown>
}

// Background detail from catalog
interface BackgroundDetail {
  name: string
  source: string
  data: Record<string, unknown>
  fluff: string | null
}

// Subclass detail from catalog
interface SubclassDetail {
  name: string
  source: string
  class_name: string
  class_source: string
  data: Record<string, unknown>
}
import {
  ALL_SKILLS,
  ABILITIES,
  getModifier,
  formatModifier,
  getProficiencyBonus,
  getTotalLevel,
  getProficienciesByType,
  isProficientInSkill,
  hasSkillExpertise,
  isProficientInSave,
  getSkillBonus,
  getSaveBonus,
  getPassivePerception,
  getArmorAC,
  getWeaponDamage,
  isFinesse,
  isRanged,
  isSpellcaster,
  getSpellcastingAbility,
  getSpellSaveDC,
  getSpellAttackBonus,
  getHitDiceString,
  formatClassString,
  getAllSpellcastingStats,
  getMulticlassCasterLevel,
} from '../../../utils/characterUtils'
import { processFormattingTags } from '../../sources/utils/textFormatting'

const route = useRoute()
const router = useRouter()
const characterStore = useCharacterStore()

// Cross-reference support
const {
  modalContent,
  lookupReference,
  handleCrossRefHover,
  handleCrossRefClick,
  hideTooltip,
  closeModal
} = useCrossReferences()

// Open modal for a class/subclass feature
const openFeatureModal = async (feature: ClassFeature) => {
  const refType = feature.subclass_name ? 'subclassFeature' : 'classFeature'
  const className = feature.subclass_name ? feature.subclass_short_name : feature.class_name

  // Show loading modal
  modalContent.value = {
    title: feature.name,
    content: '<p>Loading...</p>',
    visible: true
  }

  // Lookup the feature
  const refData = await lookupReference(refType, feature.name, feature.source, className)

  if (refData) {
    const contentData = refData.data || refData
    // Add ref_type for the renderer
    const contentWithType = { ...contentData, ref_type: refType }
    modalContent.value = {
      title: refData.name || feature.name,
      content: renderModalContent(contentWithType),
      visible: true
    }
  } else {
    modalContent.value = {
      title: feature.name,
      content: '<p>No data available for this feature.</p>',
      visible: true
    }
  }
}

// State
const characterId = computed(() => route.params.id as string)
const character = ref<Character | null>(null)
const inventory = ref<CharacterInventory[]>([])
const raceData = ref<Record<string, unknown> | null>(null)
const classData = ref<Record<string, Record<string, unknown>>>({}) // keyed by class name
const classFeatures = ref<ClassFeature[]>([])
const expandedFeatures = ref<Set<string>>(new Set()) // track which features are expanded
const featureDetails = ref<Record<string, Record<string, unknown>>>({}) // cached feature details by "name|className"
const loadingFeature = ref<string | null>(null) // currently loading feature key
const itemDetails = ref<Record<string, ItemDetail>>({}) // keyed by "name|source"
const expandedItems = ref<Set<string>>(new Set()) // track which items are expanded
const classSpells = ref<SpellInfo[]>([])
const loadingSpells = ref(false)
const expandedSpells = ref<Set<string>>(new Set())
const collapsedSpellLevels = ref<Set<number>>(new Set([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])) // start all collapsed
const backgroundDetails = ref<BackgroundDetail | null>(null)
const subclassDetails = ref<Record<string, SubclassDetail>>({}) // keyed by "className|subclassName"
const loading = ref(true)
const loadingInventory = ref(false)
const error = ref<string | null>(null)
const showInventory = ref(false)
const showPrintDialog = ref(false)
const activeTab = ref<'character' | 'equipment' | 'spells' | 'details'>('character')

// Computed properties
const totalLevel = computed(() => (character.value ? getTotalLevel(character.value) : 0))
const classString = computed(() => (character.value ? formatClassString(character.value) : ''))
const profBonus = computed(() => getProficiencyBonus(totalLevel.value))
const hitDice = computed(() => (character.value ? getHitDiceString(character.value) : '-'))
const passivePerception = computed(() =>
  character.value ? getPassivePerception(character.value) : 10
)

// Speed - from race catalog data, default 30ft
const speed = computed(() => {
  if (raceData.value && raceData.value.speed) {
    const speedData = raceData.value.speed
    // 5etools format: speed can be number or { walk: number, fly?: number, swim?: number, ... }
    if (typeof speedData === 'number') {
      return speedData
    }
    if (typeof speedData === 'object' && speedData !== null) {
      const walkSpeed = (speedData as Record<string, unknown>).walk
      if (typeof walkSpeed === 'number') return walkSpeed
    }
  }
  return 30 // Default
})

// Proficiency helpers
const armorProficiencies = computed(() =>
  character.value ? getProficienciesByType(character.value, 'armor') : []
)
const weaponProficiencies = computed(() =>
  character.value ? getProficienciesByType(character.value, 'weapon') : []
)
const toolProficiencies = computed(() =>
  character.value ? getProficienciesByType(character.value, 'tool') : []
)
const languages = computed(() =>
  character.value ? getProficienciesByType(character.value, 'language') : []
)
const hasProficiencies = computed(
  () =>
    armorProficiencies.value.length > 0 ||
    weaponProficiencies.value.length > 0 ||
    toolProficiencies.value.length > 0 ||
    languages.value.length > 0
)

// Skill/save proficiency checks
const isSkillProficient = (skillName: string) =>
  character.value ? isProficientInSkill(character.value, skillName) : false
const hasExpertise = (skillName: string) =>
  character.value ? hasSkillExpertise(character.value, skillName) : false
const isSaveProficient = (ability: string) =>
  character.value ? isProficientInSave(character.value, ability) : false

// Spellcasting
const characterIsSpellcaster = computed(() =>
  character.value ? isSpellcaster(character.value) : false
)
const spellcastingAbility = computed(() =>
  character.value ? getSpellcastingAbility(character.value) : null
)
const spellSaveDC = computed(() => (character.value ? getSpellSaveDC(character.value) : null))
const spellAttackBonus = computed(() =>
  character.value ? getSpellAttackBonus(character.value) : null
)

// All spellcasting stats for multiclass characters (one per spellcasting class)
const allSpellcastingStats = computed(() =>
  character.value ? getAllSpellcastingStats(character.value) : []
)

// Whether this is a multiclass spellcaster (has 2+ spellcasting classes)
const isMulticlassSpellcaster = computed(() => allSpellcastingStats.value.length > 1)

// Spell slots using proper multiclass caster level calculation
// Per D&D 5e rules: combine caster levels from all Spellcasting classes, then look up slots
const spellSlots = computed(() => {
  if (!character.value || !characterIsSpellcaster.value) return null

  const slots: Record<number, number> = {}

  // Check for Warlock first (uses separate Pact Magic)
  const warlock = character.value.classes?.find(
    (c) => c.class_name.toLowerCase() === 'warlock'
  )

  // Get multiclass caster level (excludes Warlock)
  const casterLevel = getMulticlassCasterLevel(character.value)

  if (casterLevel > 0) {
    // Multiclass spell slot progression (same as full caster progression)
    const multiclassSlots: Record<number, number[]> = {
      1: [2],
      2: [3],
      3: [4, 2],
      4: [4, 3],
      5: [4, 3, 2],
      6: [4, 3, 3],
      7: [4, 3, 3, 1],
      8: [4, 3, 3, 2],
      9: [4, 3, 3, 3, 1],
      10: [4, 3, 3, 3, 2],
      11: [4, 3, 3, 3, 2, 1],
      12: [4, 3, 3, 3, 2, 1],
      13: [4, 3, 3, 3, 2, 1, 1],
      14: [4, 3, 3, 3, 2, 1, 1],
      15: [4, 3, 3, 3, 2, 1, 1, 1],
      16: [4, 3, 3, 3, 2, 1, 1, 1],
      17: [4, 3, 3, 3, 2, 1, 1, 1, 1],
      18: [4, 3, 3, 3, 3, 1, 1, 1, 1],
      19: [4, 3, 3, 3, 3, 2, 1, 1, 1],
      20: [4, 3, 3, 3, 3, 2, 2, 1, 1],
    }
    const slotArray = multiclassSlots[casterLevel] || []
    slotArray.forEach((count, idx) => {
      if (count > 0) slots[idx + 1] = count
    })
  }

  // Add Warlock Pact Magic slots separately (they're tracked independently)
  if (warlock) {
    const warlockSlots: Record<number, { count: number; level: number }> = {
      1: { count: 1, level: 1 },
      2: { count: 2, level: 1 },
      3: { count: 2, level: 2 },
      4: { count: 2, level: 2 },
      5: { count: 2, level: 3 },
      6: { count: 2, level: 3 },
      7: { count: 2, level: 4 },
      8: { count: 2, level: 4 },
      9: { count: 2, level: 5 },
      10: { count: 2, level: 5 },
      11: { count: 3, level: 5 },
      12: { count: 3, level: 5 },
      13: { count: 3, level: 5 },
      14: { count: 3, level: 5 },
      15: { count: 3, level: 5 },
      16: { count: 3, level: 5 },
      17: { count: 4, level: 5 },
      18: { count: 4, level: 5 },
      19: { count: 4, level: 5 },
      20: { count: 4, level: 5 },
    }
    const pactMagic = warlockSlots[warlock.level]
    if (pactMagic) {
      // Note: Warlock pact slots are separate from regular slots
      // For now we just add them to the display at their level
      slots[pactMagic.level] = (slots[pactMagic.level] || 0) + pactMagic.count
    }
  }

  return Object.keys(slots).length > 0 ? slots : null
})

// Spells grouped by level
const spellsByLevel = computed(() => {
  const grouped: Record<number, SpellInfo[]> = {}
  for (const spell of classSpells.value) {
    if (!grouped[spell.level]) {
      grouped[spell.level] = []
    }
    grouped[spell.level].push(spell)
  }
  return grouped
})

// Spell helper functions
const getSchoolName = (code: string | null): string => {
  if (!code) return 'Unknown'
  const schools: Record<string, string> = {
    A: 'Abjuration',
    C: 'Conjuration',
    D: 'Divination',
    E: 'Enchantment',
    V: 'Evocation',
    I: 'Illusion',
    N: 'Necromancy',
    T: 'Transmutation',
  }
  return schools[code] || code
}

const getLevelDisplay = (level: number): string => {
  if (level === 0) return 'Cantrip'
  if (level === 1) return '1st Level'
  if (level === 2) return '2nd Level'
  if (level === 3) return '3rd Level'
  return `${level}th Level`
}

const toggleSpellDetails = (name: string, source: string) => {
  const key = `${name}|${source}`
  if (expandedSpells.value.has(key)) {
    expandedSpells.value.delete(key)
  } else {
    expandedSpells.value.add(key)
  }
  expandedSpells.value = new Set(expandedSpells.value)
}

const toggleSpellLevel = (level: number) => {
  if (collapsedSpellLevels.value.has(level)) {
    collapsedSpellLevels.value.delete(level)
  } else {
    collapsedSpellLevels.value.add(level)
  }
  collapsedSpellLevels.value = new Set(collapsedSpellLevels.value)
}

const isSpellLevelCollapsed = (level: number): boolean => {
  return collapsedSpellLevels.value.has(level)
}

const isSpellExpanded = (name: string, source: string): boolean => {
  return expandedSpells.value.has(`${name}|${source}`)
}

const getSpellCastingTime = (spell: SpellInfo): string => {
  const time = spell.data.time as Array<{ number: number; unit: string }> | undefined
  if (!time || time.length === 0) return 'Unknown'
  const t = time[0]
  return `${t.number} ${t.unit}`
}

const getSpellRange = (spell: SpellInfo): string => {
  const range = spell.data.range as { type: string; distance?: { type: string; amount?: number } } | undefined
  if (!range) return 'Unknown'
  if (range.type === 'point') {
    if (range.distance?.type === 'self') return 'Self'
    if (range.distance?.type === 'touch') return 'Touch'
    if (range.distance?.amount) return `${range.distance.amount} ${range.distance.type}`
  }
  if (range.type === 'special') return 'Special'
  return range.type
}

const getSpellComponents = (spell: SpellInfo): string => {
  const comp = spell.data.components as { v?: boolean; s?: boolean; m?: unknown } | undefined
  if (!comp) return ''
  const parts: string[] = []
  if (comp.v) parts.push('V')
  if (comp.s) parts.push('S')
  if (comp.m) parts.push('M')
  return parts.join(', ')
}

const getSpellDuration = (spell: SpellInfo): string => {
  const duration = spell.data.duration as Array<{ type: string; duration?: { type: string; amount: number }; concentration?: boolean }> | undefined
  if (!duration || duration.length === 0) return 'Unknown'
  const d = duration[0]
  if (d.type === 'instant') return 'Instantaneous'
  if (d.type === 'permanent') return 'Permanent'
  if (d.duration) {
    const conc = d.concentration ? 'Concentration, ' : ''
    return `${conc}${d.duration.amount} ${d.duration.type}`
  }
  return d.type
}

const getSpellDescription = (spell: SpellInfo): string => {
  const entries = spell.data.entries as unknown[] | undefined
  if (!entries) return ''

  return entries
    .map((entry) => {
      if (typeof entry === 'string') return entry
      if (typeof entry === 'object' && entry !== null) {
        const e = entry as Record<string, unknown>
        if (e.type === 'entries' && Array.isArray(e.entries)) {
          return (e.entries as unknown[])
            .filter((sub) => typeof sub === 'string')
            .join(' ')
        }
        if (e.type === 'list' && Array.isArray(e.items)) {
          return (e.items as unknown[])
            .filter((sub) => typeof sub === 'string')
            .map((s) => `• ${s}`)
            .join('\n')
        }
      }
      return ''
    })
    .filter(Boolean)
    .join('\n\n')
}

// Class Feature helpers
const toggleFeatureExpansion = async (feature: ClassFeature) => {
  const key = `${feature.name}|${feature.class_name}`

  if (expandedFeatures.value.has(key)) {
    // Collapse
    expandedFeatures.value.delete(key)
    expandedFeatures.value = new Set(expandedFeatures.value)
    return
  }

  // Expand - fetch details if not cached
  if (!featureDetails.value[key]) {
    loadingFeature.value = key
    try {
      const result = await invoke<{ success: boolean; data?: Record<string, unknown>; error?: string }>(
        'get_class_feature',
        { name: feature.name, className: feature.class_name }
      )
      if (result.success && result.data) {
        featureDetails.value = { ...featureDetails.value, [key]: result.data }
      }
    } catch (e) {
      console.error('Failed to load feature details:', e)
    } finally {
      loadingFeature.value = null
    }
  }

  expandedFeatures.value.add(key)
  expandedFeatures.value = new Set(expandedFeatures.value)
}

const isFeatureExpanded = (feature: ClassFeature): boolean => {
  return expandedFeatures.value.has(`${feature.name}|${feature.class_name}`)
}

const getFeatureDescription = (feature: ClassFeature): string => {
  const key = `${feature.name}|${feature.class_name}`
  const details = featureDetails.value[key]
  if (!details) return ''

  // 5etools format: entries array contains the description
  const entries = details.entries as unknown[] | undefined
  if (!entries) return ''

  return entries
    .map((entry) => {
      if (typeof entry === 'string') return processFormattingTags(entry)
      if (typeof entry === 'object' && entry !== null) {
        const e = entry as Record<string, unknown>
        if (e.type === 'entries' && Array.isArray(e.entries)) {
          return (e.entries as unknown[])
            .filter((sub) => typeof sub === 'string')
            .map((s) => processFormattingTags(s as string))
            .join(' ')
        }
        if (e.type === 'list' && Array.isArray(e.items)) {
          return (e.items as unknown[])
            .filter((sub) => typeof sub === 'string')
            .map((s) => `• ${processFormattingTags(s as string)}`)
            .join('\n')
        }
      }
      return ''
    })
    .filter(Boolean)
    .join('\n\n')
}

const isFeatureLoading = (feature: ClassFeature): boolean => {
  return loadingFeature.value === `${feature.name}|${feature.class_name}`
}

// Background helpers
const getBackgroundSkillProficiencies = (): string[] => {
  if (!backgroundDetails.value?.data) return []
  const skillProfs = backgroundDetails.value.data.skillProficiencies as Array<Record<string, boolean>> | undefined
  if (!skillProfs) return []
  return skillProfs.flatMap((sp) => Object.keys(sp).filter((k) => sp[k] && k !== 'choose'))
}

const getBackgroundToolProficiencies = (): string[] => {
  if (!backgroundDetails.value?.data) return []
  const toolProfs = backgroundDetails.value.data.toolProficiencies as Array<Record<string, boolean | string>> | undefined
  if (!toolProfs) return []
  const tools: string[] = []
  for (const tp of toolProfs) {
    for (const [key, val] of Object.entries(tp)) {
      if (key !== 'choose' && val) {
        tools.push(typeof val === 'string' ? val : key)
      }
    }
  }
  return tools
}

const getBackgroundLanguages = (): string => {
  if (!backgroundDetails.value?.data) return ''
  const langs = backgroundDetails.value.data.languageProficiencies as Array<{ anyStandard?: number }> | undefined
  if (!langs || langs.length === 0) return ''
  const lang = langs[0]
  if (lang.anyStandard) return `${lang.anyStandard} of your choice`
  return ''
}

const getBackgroundEquipment = (): string => {
  if (!backgroundDetails.value?.data) return ''
  const entries = backgroundDetails.value.data.entries as unknown[] | undefined
  if (!entries) return ''

  for (const entry of entries) {
    if (typeof entry === 'object' && entry !== null) {
      const e = entry as Record<string, unknown>
      if (e.type === 'list' && e.name === 'Equipment') {
        const items = e.items as string[] | undefined
        if (items) return items.join(', ')
      }
    }
  }
  return ''
}

const getBackgroundFeature = (): { name: string; description: string } | null => {
  if (!backgroundDetails.value?.data) return null
  const entries = backgroundDetails.value.data.entries as unknown[] | undefined
  if (!entries) return null

  for (const entry of entries) {
    if (typeof entry === 'object' && entry !== null) {
      const e = entry as Record<string, unknown>
      if (e.type === 'entries' && e.name && typeof e.name === 'string') {
        // Feature entries usually have a name that's the feature name
        const subEntries = e.entries as unknown[] | undefined
        if (subEntries) {
          const desc = subEntries
            .filter((se) => typeof se === 'string')
            .join(' ')
          if (desc) {
            return { name: e.name, description: desc }
          }
        }
      }
    }
  }
  return null
}

// Class mechanical info helpers
const getClassHitDice = (className: string): string => {
  const data = classData.value[className.toLowerCase()]
  if (!data) return ''
  const hd = data.hd as { faces?: number } | undefined
  return hd?.faces ? `d${hd.faces}` : ''
}

const getClassPrimaryAbility = (className: string): string => {
  const data = classData.value[className.toLowerCase()]
  if (!data) return ''

  const statNames: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma',
  }

  const primaryAbility = data.primaryAbility
  // Handle array format (newer 5etools)
  if (Array.isArray(primaryAbility)) {
    const abilities: string[] = []
    for (const ability of primaryAbility) {
      if (typeof ability === 'object' && ability !== null) {
        for (const [stat, value] of Object.entries(ability)) {
          if (value === true) abilities.push(statNames[stat] || stat.toUpperCase())
        }
      }
    }
    if (abilities.length > 0) return abilities.join(' or ')
  }
  // Handle object format
  if (typeof primaryAbility === 'object' && primaryAbility !== null) {
    const abilities: string[] = []
    for (const [stat, value] of Object.entries(primaryAbility)) {
      if (value === true) abilities.push(statNames[stat] || stat.toUpperCase())
    }
    if (abilities.length > 0) return abilities.join(' or ')
  }
  // Fallback: infer from spellcasting ability
  const spellcastingAbility = data.spellcastingAbility
  if (typeof spellcastingAbility === 'string') {
    return statNames[spellcastingAbility] || spellcastingAbility.toUpperCase()
  }
  return ''
}

const getClassSpellcastingAbility = (className: string): string => {
  const data = classData.value[className.toLowerCase()]
  if (!data) return ''
  const spellcastingAbility = data.spellcastingAbility as string | undefined
  if (!spellcastingAbility) return ''
  const abilityMap: Record<string, string> = { int: 'INT', wis: 'WIS', cha: 'CHA' }
  return abilityMap[spellcastingAbility] || spellcastingAbility.toUpperCase()
}

const getClassProficiencies = (className: string): { armor: string; weapons: string; tools: string; skills: string } => {
  const data = classData.value[className.toLowerCase()]
  const result = { armor: '', weapons: '', tools: '', skills: '' }
  if (!data) return result

  const sp = data.startingProficiencies as Record<string, unknown> | undefined
  if (!sp) return result

  // Armor - process xref tags to HTML
  if (Array.isArray(sp.armor)) {
    const armorItems = sp.armor
      .map((a: unknown) => {
        if (typeof a === 'string') return processFormattingTags(a)
        if (typeof a === 'object' && a !== null && 'proficiency' in a) {
          return processFormattingTags((a as { proficiency: string }).proficiency)
        }
        return null
      })
      .filter(Boolean)
    result.armor = armorItems.join(', ')
  }

  // Weapons - process xref tags to HTML
  if (Array.isArray(sp.weapons)) {
    const weaponItems = sp.weapons
      .map((w: unknown) => {
        if (typeof w === 'string') return processFormattingTags(w)
        if (typeof w === 'object' && w !== null && 'proficiency' in w) {
          return processFormattingTags((w as { proficiency: string }).proficiency)
        }
        return null
      })
      .filter(Boolean)
    result.weapons = weaponItems.join(', ')
  }

  // Tools
  if (Array.isArray(sp.tools)) {
    result.tools = sp.tools.length > 0 ? `${sp.tools.length} of your choice` : ''
  }

  // Skills
  if (Array.isArray(sp.skills)) {
    const skillChoices = sp.skills.map((s: unknown) => {
      if (typeof s === 'object' && s !== null && 'choose' in s) {
        const choose = s as { choose: { from?: string[]; count: number } }
        return `Choose ${choose.choose.count}`
      }
      return null
    }).filter(Boolean)
    result.skills = skillChoices.join(', ')
  }

  return result
}

const getClassSavingThrows = (className: string): string[] => {
  const data = classData.value[className.toLowerCase()]
  if (!data) return []
  const profs = data.proficiency as Array<{ [key: string]: boolean }> | undefined
  if (!Array.isArray(profs)) return []

  const statNames: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma',
  }

  const saves: string[] = []
  for (const prof of profs) {
    for (const [stat, value] of Object.entries(prof)) {
      if (value === true && statNames[stat]) {
        saves.push(statNames[stat])
      }
    }
  }
  return saves
}

// Feature display helpers
const getFeatureLevels = (className: string, subclassName?: string | null): number[] => {
  const features = classFeatures.value.filter((f) => {
    const classMatch = f.class_name?.toLowerCase() === className.toLowerCase()
    // Include class features (no subclass_name) and matching subclass features
    if (!classMatch) return false
    if (f.subclass_name) {
      return f.subclass_name === subclassName
    }
    return true
  })
  const levels = [...new Set(features.map((f) => f.level))].sort((a, b) => a - b)
  return levels
}

const getFeaturesAtLevel = (className: string, subclassName: string | null | undefined, level: number): ClassFeature[] => {
  return classFeatures.value.filter((f) => {
    const classMatch = f.class_name?.toLowerCase() === className.toLowerCase()
    const levelMatch = f.level === level
    if (!classMatch || !levelMatch) return false
    // Include class features (no subclass_name) and matching subclass features
    if (f.subclass_name) {
      return f.subclass_name === subclassName
    }
    return true
  })
}

const formatOrdinal = (n: number): string => {
  const suffixes = ['th', 'st', 'nd', 'rd']
  const v = n % 100
  return n + (suffixes[(v - 20) % 10] || suffixes[v] || suffixes[0])
}

const getSubclassDescription = (className: string, subclassName: string): string => {
  const key = `${className}|${subclassName}`
  const subclass = subclassDetails.value[key]
  if (!subclass?.data) return ''

  const entries = subclass.data.entries as unknown[] | undefined
  if (!entries) return ''

  // Get first paragraph of description
  for (const entry of entries) {
    if (typeof entry === 'string') return entry
    if (typeof entry === 'object' && entry !== null) {
      const e = entry as Record<string, unknown>
      if (e.type === 'entries' && Array.isArray(e.entries)) {
        const firstText = (e.entries as unknown[]).find((sub) => typeof sub === 'string')
        if (firstText) return firstText as string
      }
    }
  }
  return ''
}

// Equipment
const equippedItems = computed(() => inventory.value.filter((i) => i.equipped !== 0))
const equippedArmor = computed(() =>
  equippedItems.value.find((i) => {
    const name = i.item_name.toLowerCase()
    return (
      name.includes('armor') ||
      name.includes('mail') ||
      name.includes('hide') ||
      name.includes('leather') ||
      name.includes('plate') ||
      name.includes('robe')
    )
  })
)
const equippedWeapons = computed(() =>
  equippedItems.value.filter((i) => {
    const name = i.item_name.toLowerCase()
    return (
      name.includes('sword') ||
      name.includes('axe') ||
      name.includes('bow') ||
      name.includes('dagger') ||
      name.includes('mace') ||
      name.includes('staff') ||
      name.includes('crossbow') ||
      name.includes('spear') ||
      name.includes('hammer')
    )
  })
)

// AC calculation
const baseAC = computed(() => {
  if (!character.value) return 10
  const dexMod = getModifier(character.value.dexterity)

  if (equippedArmor.value) {
    return getArmorAC(equippedArmor.value.item_name, dexMod)
  }

  // Unarmored: 10 + DEX
  return 10 + dexMod
})

// Attacks from equipped weapons
const attacks = computed(() => {
  if (!character.value || equippedWeapons.value.length === 0) return []

  const strMod = getModifier(character.value.strength)
  const dexMod = getModifier(character.value.dexterity)
  const prof = profBonus.value

  return equippedWeapons.value.map((weapon) => {
    let abilityMod = strMod
    if (isRanged(weapon.item_name)) {
      abilityMod = dexMod
    } else if (isFinesse(weapon.item_name) && dexMod > strMod) {
      abilityMod = dexMod
    }

    return {
      name: weapon.item_name,
      attackBonus: prof + abilityMod,
      damage: getWeaponDamage(weapon.item_name, abilityMod),
    }
  })
})

// Personality and NPC checks
const hasPersonality = computed(() => {
  if (!character.value) return false
  return (
    character.value.traits ||
    character.value.ideals ||
    character.value.bonds ||
    character.value.flaws
  )
})

const hasNpcInfo = computed(() => {
  if (!character.value) return false
  return character.value.role || character.value.location || character.value.faction
})

// Methods
const formatMod = (mod: number) => formatModifier(mod)

const loadCharacter = async () => {
  loading.value = true
  error.value = null

  try {
    character.value = await characterStore.getCharacter(characterId.value)
    if (!character.value) {
      error.value = 'Character not found'
    } else {
      // Load inventory and catalog data in parallel
      await Promise.all([
        loadInventory(),
        loadRaceData(),
        loadClassData(),
        loadClassSpells(),
        loadBackgroundDetails(),
        loadSubclassDetails(),
      ])
      // Load features after class data is ready (it parses from classData)
      loadClassFeatures()
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load character'
  } finally {
    loading.value = false
  }
}

const loadRaceData = async () => {
  if (!character.value?.race_name) return

  try {
    const source = character.value.race_source || 'PHB'
    const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
      'get_race_by_name',
      { name: character.value.race_name, source }
    )
    if (result.success && result.data) {
      raceData.value = result.data
    }
  } catch (e) {
    console.error('Failed to load race data:', e)
  }
}

const loadClassData = async () => {
  if (!character.value?.classes?.length) return

  try {
    const results = await Promise.all(
      character.value.classes.map(async (cls) => {
        const source = cls.class_source || 'PHB'
        const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
          'get_class_by_name',
          { name: cls.class_name, source }
        )
        return { className: cls.class_name, data: result.success ? result.data : null }
      })
    )

    const newClassData: Record<string, Record<string, unknown>> = {}
    for (const { className, data } of results) {
      if (data) {
        newClassData[className.toLowerCase()] = data
      }
    }
    classData.value = newClassData
  } catch (e) {
    console.error('Failed to load class data:', e)
  }
}

const loadClassFeatures = () => {
  if (!character.value?.classes?.length) return

  const allFeatures: ClassFeature[] = []

  for (const cls of character.value.classes) {
    const data = classData.value[cls.class_name.toLowerCase()]
    if (!data) continue

    // Parse classFeatures from the class data (format: "FeatureName|ClassName|ClassSource|Level" or object)
    const rawFeatures = data.classFeatures as Array<string | { classFeature: string }> | undefined
    if (rawFeatures) {
      for (const feature of rawFeatures) {
        let featureStr = ''
        if (typeof feature === 'string') {
          featureStr = feature
        } else if (typeof feature === 'object' && feature.classFeature) {
          featureStr = feature.classFeature
        }

        if (featureStr) {
          const parts = featureStr.split('|')
          if (parts.length >= 4) {
            const featureName = parts[0]
            const className = parts[1] || cls.class_name
            const classSource = parts[2] || cls.class_source || 'PHB'
            const level = parseInt(parts[3]) || 1

            // Only include features up to character's current level in this class
            if (level <= cls.level) {
              allFeatures.push({
                name: featureName,
                source: classSource,
                class_name: className,
                class_source: classSource,
                level: level,
                data: '',
              })
            }
          }
        }
      }
    }

    // Also load subclass features if character has a subclass
    if (cls.subclass_name) {
      const subclassKey = `${cls.class_name}|${cls.subclass_name}`
      const subclass = subclassDetails.value[subclassKey]
      if (subclass?.data) {
        // Subclass features format: "FeatureName|ClassName|ClassSource|SubclassShortName|SubclassSource|Level" (6 parts)
        const rawSubFeatures = (subclass.data as Record<string, unknown>).subclassFeatures as string[] | undefined
        if (rawSubFeatures) {
          for (const featureRef of rawSubFeatures) {
            if (typeof featureRef !== 'string') continue
            const parts = featureRef.split('|')
            if (parts.length >= 6) {
              const featureName = parts[0]
              const className = parts[1] || cls.class_name
              const classSource = parts[2] || cls.class_source || 'PHB'
              const subclassShortName = parts[3]
              const subclassSource = parts[4]
              const level = parseInt(parts[5]) || 1

              // Only include features up to character's current level
              if (level <= cls.level) {
                allFeatures.push({
                  name: featureName,
                  source: subclassSource || classSource,
                  class_name: className,
                  class_source: classSource,
                  level: level,
                  data: '',
                  subclass_name: cls.subclass_name,
                  subclass_short_name: subclassShortName,
                  subclass_source: subclassSource,
                })
              }
            }
          }
        }
      }
    }
  }

  // Sort by level, then by subclass (class features first), then by name
  allFeatures.sort((a, b) => {
    if (a.level !== b.level) return a.level - b.level
    // Class features before subclass features at same level
    if (a.subclass_name && !b.subclass_name) return 1
    if (!a.subclass_name && b.subclass_name) return -1
    return a.name.localeCompare(b.name)
  })

  classFeatures.value = allFeatures
}

const loadClassSpells = async () => {
  if (!character.value?.classes?.length) return

  // Find spellcasting classes
  const spellcastingClasses = ['bard', 'cleric', 'druid', 'paladin', 'ranger', 'sorcerer', 'warlock', 'wizard']
  const charSpellClasses = character.value.classes.filter((c) =>
    spellcastingClasses.includes(c.class_name.toLowerCase())
  )

  if (charSpellClasses.length === 0) return

  loadingSpells.value = true

  try {
    // Determine max spell level based on class and level
    const getMaxSpellLevel = (className: string, level: number): number => {
      const lowerName = className.toLowerCase()
      if (['paladin', 'ranger'].includes(lowerName)) {
        // Half casters - spell level = floor((level + 1) / 2) but max 5
        if (level < 2) return 0
        return Math.min(5, Math.ceil((level - 1) / 2))
      }
      if (lowerName === 'warlock') {
        // Warlock pact magic
        if (level < 1) return 0
        if (level < 3) return 1
        if (level < 5) return 2
        if (level < 7) return 3
        if (level < 9) return 4
        return 5
      }
      // Full casters
      return Math.min(9, Math.ceil(level / 2))
    }

    let maxLevel = 0
    for (const cls of charSpellClasses) {
      maxLevel = Math.max(maxLevel, getMaxSpellLevel(cls.class_name, cls.level))
    }

    // Use the new get_spells_by_class command which joins with spell_classes table
    // Fetch spells for each spellcasting class the character has
    const spellsByClassName = new Map<string, SpellInfo[]>()

    for (const cls of charSpellClasses) {
      const result = await invoke<{ success: boolean; data?: Array<Record<string, unknown>>; error?: string }>(
        'get_spells_by_class',
        { className: cls.class_name }
      )

      if (result.success && result.data) {
        const classSpellList: SpellInfo[] = []
        for (const rawSpell of result.data) {
          // Filter by max spell level for this character
          const spellLevel = rawSpell.level as number
          if (spellLevel > maxLevel) continue

          // Backend merges data at top level via entity_to_json
          // Store the whole rawSpell object as data for accessing time, range, etc.
          classSpellList.push({
            name: rawSpell.name as string,
            source: rawSpell.source as string,
            level: spellLevel,
            school: rawSpell.school as string | null,
            ritual: (rawSpell.ritual as number) === 1 || rawSpell.ritual === true,
            concentration: (rawSpell.concentration as number) === 1 || rawSpell.concentration === true,
            data: rawSpell as Record<string, unknown>,
          })
        }
        spellsByClassName.set(cls.class_name, classSpellList)
      }
    }

    // Merge and deduplicate spells from all classes
    // Keep name|source as key since different sources may have different spell versions
    const seenSpells = new Set<string>()
    const allSpells: SpellInfo[] = []

    for (const spellList of spellsByClassName.values()) {
      for (const spell of spellList) {
        const key = `${spell.name}|${spell.source}`
        if (!seenSpells.has(key)) {
          seenSpells.add(key)
          allSpells.push(spell)
        }
      }
    }

    // Sort by level, then name
    allSpells.sort((a, b) => {
      if (a.level !== b.level) return a.level - b.level
      return a.name.localeCompare(b.name)
    })

    classSpells.value = allSpells
  } catch (e) {
    console.error('Failed to load class spells:', e)
    classSpells.value = []
  } finally {
    loadingSpells.value = false
  }
}

const loadBackgroundDetails = async () => {
  if (!character.value?.background_name) return

  try {
    const source = character.value.background_source || 'PHB'
    const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
      'get_background_by_name',
      { name: character.value.background_name, source }
    )

    if (result.success && result.data) {
      const rawBg = result.data as unknown as {
        name: string
        source: string
        data: string | Record<string, unknown>
        fluff: string | null
      }
      backgroundDetails.value = {
        name: rawBg.name,
        source: rawBg.source,
        data: typeof rawBg.data === 'string' ? JSON.parse(rawBg.data) : rawBg.data,
        fluff: rawBg.fluff,
      }
    }
  } catch (e) {
    console.error('Failed to load background details:', e)
  }
}

const loadSubclassDetails = async () => {
  if (!character.value?.classes?.length) return

  try {
    for (const cls of character.value.classes) {
      if (!cls.subclass_name) continue

      const source = cls.subclass_source || 'PHB'
      const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
        'get_subclass_by_name',
        { name: cls.subclass_name, className: cls.class_name, source }
      )

      if (result.success && result.data) {
        // result.data IS the parsed 5etools data (entity_to_json merges it)
        const subclassData = result.data as Record<string, unknown>
        const key = `${cls.class_name}|${cls.subclass_name}`
        subclassDetails.value[key] = {
          name: (subclassData.name as string) || cls.subclass_name,
          source: (subclassData.source as string) || cls.subclass_source || 'PHB',
          class_name: (subclassData.className as string) || cls.class_name,
          class_source: (subclassData.classSource as string) || 'PHB',
          data: subclassData, // The full 5etools data is already here
        }
      }
    }
  } catch (e) {
    console.error('Failed to load subclass details:', e)
  }
}

const loadInventory = async () => {
  if (!characterId.value) return

  loadingInventory.value = true
  try {
    const result = await invoke<{ data: CharacterInventory[] }>('get_character_inventory', {
      characterId: characterId.value,
    })
    inventory.value = result.data || []
  } catch (e) {
    console.error('Failed to load inventory:', e)
    inventory.value = []
  } finally {
    loadingInventory.value = false
  }
}

const goBack = () => {
  router.back()
}

const openInventory = () => {
  showInventory.value = true
}

const printCharacter = () => {
  showPrintDialog.value = true
}

// Item details
const getItemKey = (name: string, source: string) => `${name}|${source}`

const isItemExpanded = (name: string, source: string) => {
  return expandedItems.value.has(getItemKey(name, source))
}

const toggleItemDetails = async (name: string, source: string) => {
  const key = getItemKey(name, source)

  if (expandedItems.value.has(key)) {
    expandedItems.value.delete(key)
    expandedItems.value = new Set(expandedItems.value) // trigger reactivity
    return
  }

  // Fetch item details if not already loaded
  if (!itemDetails.value[key]) {
    try {
      const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
        'get_item_by_name',
        { name, source }
      )
      if (result.success && result.data) {
        // Parse the data field if it's a string
        const rawItem = result.data as unknown as {
          name: string
          source: string
          item_type: string | null
          rarity: string | null
          data: string | Record<string, unknown>
          fluff: string | null
        }
        const item: ItemDetail = {
          name: rawItem.name,
          source: rawItem.source,
          item_type: rawItem.item_type,
          rarity: rawItem.rarity,
          data: typeof rawItem.data === 'string' ? JSON.parse(rawItem.data) : rawItem.data,
          fluff: rawItem.fluff,
        }
        itemDetails.value[key] = item
      }
    } catch (e) {
      console.error('Failed to load item details:', e)
      return
    }
  }

  expandedItems.value.add(key)
  expandedItems.value = new Set(expandedItems.value) // trigger reactivity
}

const getItemDetail = (name: string, source: string): ItemDetail | null => {
  return itemDetails.value[getItemKey(name, source)] || null
}

// Helper to extract item description from 5etools data
const getItemDescription = (item: ItemDetail): string => {
  if (!item.data) return ''

  const entries = item.data.entries as unknown[]
  if (!entries || !Array.isArray(entries)) return ''

  // Flatten entries to text
  return entries
    .map((entry) => {
      if (typeof entry === 'string') return entry
      if (typeof entry === 'object' && entry !== null) {
        const e = entry as Record<string, unknown>
        if (e.type === 'entries' && Array.isArray(e.entries)) {
          return (e.entries as unknown[])
            .filter((sub) => typeof sub === 'string')
            .join(' ')
        }
        if (e.type === 'list' && Array.isArray(e.items)) {
          return (e.items as unknown[])
            .filter((sub) => typeof sub === 'string')
            .join(', ')
        }
      }
      return ''
    })
    .filter(Boolean)
    .join(' ')
}

// Helper to get item properties
const getItemProperties = (item: ItemDetail): string[] => {
  if (!item.data) return []

  const props: string[] = []
  const data = item.data

  // Weapon properties
  if (data.property && Array.isArray(data.property)) {
    const propMap: Record<string, string> = {
      'F': 'Finesse',
      'H': 'Heavy',
      'L': 'Light',
      'R': 'Reach',
      'T': 'Thrown',
      '2H': 'Two-Handed',
      'V': 'Versatile',
      'A': 'Ammunition',
      'LD': 'Loading',
      'S': 'Special',
    }
    for (const p of data.property as string[]) {
      if (propMap[p]) props.push(propMap[p])
    }
  }

  // Armor properties
  if (data.stealth) props.push('Stealth Disadvantage')
  if (data.strength) props.push(`Str ${data.strength}+ required`)

  // Magic item properties
  if (data.reqAttune) {
    if (data.reqAttune === true) props.push('Requires Attunement')
    else if (typeof data.reqAttune === 'string') props.push(`Attunement: ${data.reqAttune}`)
  }

  return props
}

onMounted(() => {
  loadCharacter()

  // Set up cross-reference event listeners
  document.addEventListener('mouseover', handleCrossRefHover as any)
  document.addEventListener('mouseout', (e) => {
    const target = e.target as HTMLElement
    if (target.classList?.contains('cross-ref-link')) {
      hideTooltip()
    }
  })
  document.addEventListener('click', handleCrossRefClick as any)
})

onUnmounted(() => {
  // Clean up cross-reference event listeners
  document.removeEventListener('mouseover', handleCrossRefHover as any)
  document.removeEventListener('click', handleCrossRefClick as any)
})
</script>

<style scoped>
.character-sheet {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--spacing-lg);
}

.loading-state,
.error-state {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

/* Header */
.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-lg);
  border-bottom: 2px solid var(--color-border);
}

.header-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.btn-back {
  align-self: flex-start;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: transparent;
  border: 1px solid #ccc;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  margin-bottom: var(--spacing-sm);
}

.btn-back:hover {
  background: var(--color-surface-variant);
}

.character-name {
  font-size: 2rem;
  font-weight: bold;
  color: var(--color-text);
  margin: 0;
}

.character-subtitle {
  color: var(--color-text-secondary);
  font-size: 1.1rem;
}

.character-background {
  color: var(--color-text-secondary);
  font-size: 0.9rem;
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
  align-items: center;
}

.npc-badge {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning, #d97706);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.player-name {
  font-style: italic;
  color: var(--color-text-secondary);
}

/* Tab Navigation */
.tab-navigation {
  display: flex;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
}

.tab-button {
  padding: var(--spacing-sm) var(--spacing-lg);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.tab-button:hover {
  color: var(--color-text);
}

.tab-button.active {
  color: var(--color-primary-600);
  border-bottom-color: var(--color-primary-600);
}

/* Content Layouts */
.sheet-content.three-columns {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--spacing-lg);
}

.sheet-content.single-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 700px;
}

.sheet-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

/* Sections */
.sheet-section {
  background: var(--color-surface);
  border: 1px solid #ccc;
  border-radius: var(--radius-lg);
  padding: var(--spacing-md);
}

.sheet-section h2 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-md) 0;
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.section-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.section-header-row h2 {
  margin: 0;
  padding: 0;
  border: none;
}

/* Ability Scores */
.ability-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-sm);
}

.ability-box {
  text-align: center;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.ability-name {
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ability-value {
  font-size: 1.25rem;
  font-weight: bold;
  color: var(--color-text);
}

.ability-modifier {
  font-size: 0.9rem;
  color: var(--color-primary-600);
  font-weight: 500;
}

/* Combat Stats */
.combat-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-sm);
}

.combat-stat {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.stat-label {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.stat-value {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text);
}

.stat-note {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

/* Saves */
.saves-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.save-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
}

.save-item:hover {
  background: var(--color-surface-variant);
}

.save-proficient {
  width: 16px;
  color: var(--color-text-secondary);
  opacity: 0.3;
}

.save-proficient.active {
  color: var(--color-primary-600);
  opacity: 1;
}

.save-name {
  flex: 1;
  text-transform: capitalize;
  font-size: 0.9rem;
}

.save-bonus {
  font-weight: 600;
  color: var(--color-text);
}

/* Skills */
.skills-section {
  max-height: 500px;
  overflow-y: auto;
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.skill-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
}

.skill-item:hover {
  background: var(--color-surface-variant);
}

.skill-proficient {
  width: 20px;
  color: var(--color-text-secondary);
  opacity: 0.3;
}

.skill-proficient.active {
  color: var(--color-primary-600);
  opacity: 1;
}

.skill-proficient.expertise {
  color: var(--color-success, #059669);
}

.skill-name {
  flex: 1;
}

.skill-ability {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.skill-bonus {
  font-weight: 600;
  min-width: 30px;
  text-align: right;
}

/* Attacks */
.attacks-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.attack-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.attack-name {
  flex: 1;
  font-weight: 500;
}

.attack-bonus {
  font-weight: 600;
  color: var(--color-primary-600);
}

.attack-damage {
  font-size: 0.9rem;
  color: var(--color-text-secondary);
}

/* Proficiencies */
.proficiency-group {
  margin-bottom: var(--spacing-sm);
  font-size: 0.9rem;
}

.proficiency-group:last-child {
  margin-bottom: 0;
}

.empty-proficiencies {
  color: var(--color-text-secondary);
  font-style: italic;
  font-size: 0.9rem;
}

/* Class Features */
.features-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  max-height: 400px;
  overflow-y: auto;
}

.feature-item {
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  overflow: hidden;
}

.feature-item.expanded {
  background: var(--color-surface);
  border: 1px solid #ccc;
}

.feature-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.feature-header:hover {
  background: var(--color-surface-hover);
}

.feature-name {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-weight: 500;
}

.feature-name .expand-icon {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  width: 12px;
}

.feature-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--color-text-secondary);
  font-size: 0.8rem;
}

.subclass-badge {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.7rem;
  font-weight: 600;
}

.theme-dark .subclass-badge {
  background: var(--color-primary-900);
  color: var(--color-primary-300);
}

.feature-details {
  padding: var(--spacing-sm) var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.feature-loading {
  color: var(--color-text-secondary);
  font-style: italic;
}

.feature-description {
  line-height: 1.5;
  color: var(--color-text);
  white-space: pre-wrap;
}

.feature-no-desc {
  color: var(--color-text-secondary);
  font-style: italic;
}

.feature-name {
  font-weight: 500;
}

.feature-meta {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

/* Spellcasting */
.spell-stats {
  display: flex;
  gap: var(--spacing-md);
}

.spell-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  flex: 1;
}

.spell-ability {
  font-size: 0.9rem;
}

.spell-note {
  margin-top: var(--spacing-sm);
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.spell-info-note {
  color: var(--color-text-secondary);
  font-style: italic;
  margin-bottom: var(--spacing-md);
}

/* Spell List */
.spell-list-container {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.spell-level-group {
  margin-bottom: var(--spacing-md);
}

.spell-level-header {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-primary-600);
  margin-bottom: var(--spacing-sm);
  padding-bottom: var(--spacing-xs);
  border-bottom: 2px solid var(--color-primary-200);
}

.spell-level-header.collapsible {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  user-select: none;
  transition: color 0.15s ease;
}

.spell-level-header.collapsible:hover {
  color: var(--color-primary-700);
}

.spell-level-header .collapse-icon {
  font-size: 0.75rem;
  width: 1rem;
  text-align: center;
  transition: transform 0.15s ease;
}

.spell-level-header .spell-count {
  font-weight: 400;
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin-left: auto;
}

.spell-level-header.collapsed {
  margin-bottom: 0;
  border-bottom-color: var(--color-border);
}

.spell-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.spell-card {
  background: var(--color-surface-variant);
  border: 1px solid #ccc;
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: all 0.2s ease;
}

.spell-card.expanded {
  border-color: var(--color-primary-300);
}

.spell-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background 0.15s ease;
}

.spell-card-header:hover {
  background: var(--color-surface-hover);
}

.spell-card-header .spell-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.spell-tag {
  font-size: 0.65rem;
  font-weight: 600;
  padding: 1px 4px;
  border-radius: 3px;
  text-transform: uppercase;
}

.spell-tag.ritual {
  background: var(--color-success-100);
  color: var(--color-success-700);
}

.spell-tag.conc {
  background: var(--color-warning-100);
  color: var(--color-warning-700);
}

.spell-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
}

.spell-school {
  color: var(--color-text-secondary);
}

.spell-card-details {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
  font-size: 0.9rem;
}

.spell-stats-mini {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-xs) var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.spell-stat-mini {
  display: flex;
  gap: var(--spacing-xs);
}

.spell-stat-mini .label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.spell-description {
  line-height: 1.5;
  white-space: pre-wrap;
  color: var(--color-text);
}

/* Spells Tab */
.spell-stats-row {
  display: flex;
  gap: var(--spacing-lg);
  justify-content: center;
}

.spell-stats-row.multiclass {
  justify-content: flex-start;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  margin-bottom: var(--spacing-sm);
}

.spell-stats-row.multiclass .spell-stat-box {
  background: var(--color-surface);
  min-width: 80px;
  padding: var(--spacing-sm) var(--spacing-md);
}

.spell-stats-row.multiclass .spell-stat-box .stat-value.large {
  font-size: 1.25rem;
}

.spell-class-label {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--color-primary-500);
  min-width: 80px;
  display: flex;
  align-items: center;
}

.spell-stat-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  min-width: 100px;
}

.spell-stat-box .stat-value.large {
  font-size: 1.75rem;
  font-weight: 700;
}

.spell-slots-grid {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.spell-slot-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) 0;
}

.slot-level {
  font-weight: 600;
  min-width: 70px;
  color: var(--color-text-secondary);
}

.cantrip-row {
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-xs);
  padding-bottom: var(--spacing-sm);
}

.slot-unlimited {
  font-size: 0.85rem;
  font-style: italic;
  color: var(--color-text-tertiary);
}

.slot-boxes {
  display: flex;
  gap: var(--spacing-xs);
}

.slot-box {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-primary-500);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
}

.slot-count {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
}

/* Personality */
.personality-item {
  margin-bottom: var(--spacing-sm);
  font-size: 0.9rem;
}

.personality-item:last-child {
  margin-bottom: 0;
}

/* Item Cards */
.item-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.item-card {
  background: var(--color-surface-variant);
  border: 1px solid #ccc;
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: all 0.2s ease;
}

.item-card.expanded {
  border-color: var(--color-primary-300);
}

.item-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background 0.15s ease;
}

.item-card-header:hover {
  background: var(--color-surface-hover);
}

.item-card-header .item-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.item-card-header .item-qty {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-weight: normal;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
}

.item-source {
  color: var(--color-text-secondary);
}

.item-equipped-badge {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
}

.item-attuned {
  background: var(--color-warning-100);
  color: var(--color-warning-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
}

.expand-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  font-weight: bold;
  color: var(--color-text-secondary);
}

.item-card-details {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
  font-size: 0.9rem;
}

.item-detail-row {
  display: flex;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.detail-label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.detail-value.rarity {
  text-transform: capitalize;
}

.detail-value.rarity.common {
  color: var(--color-text-secondary);
}

.detail-value.rarity.uncommon {
  color: #16a34a;
}

.detail-value.rarity.rare {
  color: #2563eb;
}

.detail-value.rarity.very.rare {
  color: #7c3aed;
}

.detail-value.rarity.legendary {
  color: #ea580c;
}

.detail-value.rarity.artifact {
  color: #dc2626;
}

.item-description {
  margin-top: var(--spacing-sm);
  color: var(--color-text);
  line-height: 1.5;
}

.item-notes {
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px dashed var(--color-border);
  font-style: italic;
  color: var(--color-text-secondary);
}

.loading-details {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Currency */
.currency-display {
  display: flex;
  gap: var(--spacing-lg);
  justify-content: center;
}

.currency-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.currency-item.large .currency-value {
  font-size: 1.5rem;
}

.currency-icon {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
}

.currency-icon.pp {
  background: #e0e7ff;
  color: #4338ca;
}

.currency-icon.gp {
  background: #fef3c7;
  color: #d97706;
}

.currency-icon.ep,
.currency-icon.sp {
  background: #f3f4f6;
  color: #6b7280;
}

.currency-icon.cp {
  background: #fef2f2;
  color: #dc2626;
}

.currency-value {
  font-size: 1.1rem;
  font-weight: bold;
}

/* Details Tab */
.details-card {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.details-card h3 {
  font-size: 0.95rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-primary-600);
}

.details-card.feature-card {
  border-left: 3px solid var(--color-primary-500);
}

.proficiency-grid {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.prof-item {
  display: flex;
  gap: var(--spacing-sm);
}

.prof-label {
  font-weight: 500;
  color: var(--color-text-secondary);
  min-width: 80px;
}

.loading-text {
  color: var(--color-text-secondary);
  font-style: italic;
}

.npc-details-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: var(--spacing-md);
}

.npc-detail-item {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.npc-label {
  font-size: 0.8rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--color-text-secondary);
}

.npc-value {
  font-size: 1rem;
  font-weight: 500;
}

.class-details-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.class-detail-card {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
  border: 1px solid #ccc;
}

.class-detail-card .class-header {
  margin-bottom: var(--spacing-md);
}

.class-detail-card .class-header h3 {
  font-size: 1.1rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.class-level-badge {
  font-size: 0.8rem;
  font-weight: 500;
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.class-stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-md);
}

.class-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--spacing-sm);
  background: var(--color-surface);
  border-radius: var(--radius-sm);
}

.class-stat-label {
  font-size: 0.7rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--color-text-secondary);
  letter-spacing: 0.5px;
}

.class-stat-value {
  font-size: 0.9rem;
  font-weight: 600;
}

.class-stat-value.spellcaster {
  color: var(--color-primary-500);
}

.class-proficiencies {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.class-proficiencies h4 {
  font-size: 0.85rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-text-secondary);
}

/* Links in proficiencies */
.class-proficiencies :deep(a) {
  color: var(--color-primary-500);
  text-decoration: none;
}

.class-proficiencies :deep(a:hover) {
  text-decoration: underline;
}

.subclass-section {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-md);
  border-left: 3px solid var(--color-secondary-500);
}

.subclass-header {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--color-secondary-600);
  margin-bottom: var(--spacing-sm);
}

.subclass-description {
  color: var(--color-text-secondary);
  font-size: 0.85rem;
  line-height: 1.5;
}

.class-features-section {
  margin-top: var(--spacing-md);
}

.class-features-section h4 {
  font-size: 0.9rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-text-secondary);
}

.features-by-level {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.feature-level-group {
  display: flex;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
  line-height: 1.4;
}

.level-label {
  font-weight: 600;
  min-width: 85px;
  color: var(--color-text-secondary);
}

.feature-links {
  flex: 1;
}

.feature-links a {
  color: var(--color-primary-500);
  text-decoration: none;
}

.feature-links a:hover {
  text-decoration: underline;
}

.feature-links a.subclass-feature {
  color: var(--color-secondary-500, #9c27b0);
  font-style: italic;
}

.feature-links a.subclass-feature:hover {
  color: var(--color-secondary-600, #7b1fa2);
}

/* Inventory */
.equipped-list,
.inventory-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.equipped-item,
.inventory-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.item-name {
  flex: 1;
  font-weight: 500;
}

.item-source {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.item-qty {
  font-size: 0.9rem;
  color: var(--color-text-secondary);
}

.item-equipped {
  font-size: 0.75rem;
  padding: 2px 6px;
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  border-radius: var(--radius-sm);
}

/* Classes */
.class-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.class-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.class-name {
  font-weight: 600;
}

.class-subclass {
  color: var(--color-text-secondary);
}

.class-level {
  margin-left: auto;
  font-size: 0.9rem;
  color: var(--color-primary-600);
}

/* Details */
.detail-item {
  margin-bottom: var(--spacing-sm);
}

.detail-item:last-child {
  margin-bottom: 0;
}

/* Empty states */
.empty-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-style: italic;
}

.loading-inventory {
  text-align: center;
  padding: var(--spacing-md);
  color: var(--color-text-secondary);
}

/* Responsive */
@media (max-width: 900px) {
  .sheet-content.three-columns {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 600px) {
  .sheet-content.three-columns {
    grid-template-columns: 1fr;
  }

  .ability-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .combat-grid {
    grid-template-columns: 1fr;
  }

  .currency-display {
    flex-wrap: wrap;
    gap: var(--spacing-md);
  }

  .sheet-header {
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .header-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>

<template>
  <MainLayout>
    <div class="character-sheet-view">
      <div v-if="characterStore.loading" class="loading">
        Loading character...
      </div>

      <div v-else-if="characterStore.error" class="error-message">
        {{ characterStore.error }}
      </div>

      <div v-else-if="!character || !data" class="error-message">
        Character not found
      </div>

      <template v-else-if="data">
        <!-- Header -->
        <div class="sheet-header">
          <div class="header-main">
            <button @click="goBack" class="btn-back">Back</button>
            <!-- Edit mode header -->
            <template v-if="isEditing && editData">
              <input
                v-model="editData.character_name"
                class="edit-input edit-name"
                placeholder="Character Name"
              />
              <div class="character-subtitle">
                Level {{ data.level }} {{ data.race }}{{ data.subrace ? ` (${data.subrace})` : '' }} {{ classString }}
              </div>
              <div class="edit-alignment">
                <span>{{ data.background }} | </span>
                <input
                  v-model="editData.alignment"
                  class="edit-input edit-alignment-input"
                  placeholder="Alignment"
                />
              </div>
            </template>
            <!-- View mode header -->
            <template v-else>
              <h1 class="character-name">{{ data.character_name }}</h1>
              <div class="character-subtitle">
                Level {{ data.level }} {{ data.race }}{{ data.subrace ? ` (${data.subrace})` : '' }} {{ classString }}
              </div>
              <div v-if="data.background" class="character-background">
                {{ data.background }}{{ data.alignment ? ` | ${data.alignment}` : '' }}
              </div>
            </template>
          </div>
          <div class="header-actions">
            <template v-if="isEditing">
              <button @click="saveEdits" class="btn-primary">Save</button>
              <button @click="cancelEditing" class="btn-secondary">Cancel</button>
            </template>
            <template v-else>
              <button @click="startEditing" class="btn-secondary">Edit</button>
              <button @click="printCharacter" class="btn-secondary" :disabled="isPrintingPdf">
                {{ isPrintingPdf ? 'Generating...' : 'Print PDF' }}
              </button>
              <button @click="levelUp" class="btn-secondary">Level Up</button>
              <button @click="deleteCharacter" class="btn-danger">Delete</button>
            </template>
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
            v-if="isSpellcaster"
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

        <!-- Character Tab Content -->
        <div v-if="activeTab === 'character'" class="sheet-content">
          <!-- Left Column: Abilities & Combat -->
          <div class="sheet-column">
            <!-- Ability Scores -->
            <section class="sheet-section">
              <h2 class="section-title">Ability Scores</h2>
              <div class="ability-grid">
                <div v-for="(score, ability) in data.abilities" :key="ability" class="ability-box">
                  <div class="ability-name">{{ ability.slice(0, 3).toUpperCase() }}</div>
                  <template v-if="isEditing && editData">
                    <input
                      type="number"
                      v-model.number="(editData.abilities as Record<string, number>)[ability as string]"
                      :min="1"
                      :max="30"
                      class="edit-input edit-ability"
                    />
                    <div class="ability-modifier">{{ formatModifier(getModifier((editData.abilities as Record<string, number>)[ability as string])) }}</div>
                  </template>
                  <template v-else>
                    <div class="ability-score">{{ score }}</div>
                    <div class="ability-modifier">{{ formatModifier(getModifier(score)) }}</div>
                  </template>
                </div>
              </div>
            </section>

            <!-- Combat Stats -->
            <section class="sheet-section">
              <h2 class="section-title">Combat</h2>
              <div class="combat-grid">
                <div class="combat-stat">
                  <span class="stat-label">Armor Class</span>
                  <span class="stat-value">
                    {{ baseAC }}<span v-if="hasShield" class="ac-shield"> / {{ baseAC + 2 }}</span>
                  </span>
                  <span v-if="data.equipped.armor" class="stat-note">{{ data.equipped.armor }}</span>
                  <span v-if="hasShield" class="stat-note">w/o / w shield</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Initiative</span>
                  <span class="stat-value">{{ formatModifier(getModifier(data.abilities.dexterity)) }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Speed</span>
                  <span class="stat-value">{{ data.speed }} ft</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Passive Perception</span>
                  <span class="stat-value">{{ passivePerception }}</span>
                </div>
                <div class="combat-stat hp-stat">
                  <span class="stat-label">Hit Points</span>
                  <span v-if="isEditing && editData" class="stat-value hp-edit-group">
                    <input
                      type="number"
                      v-model.number="editData.current_hp"
                      :min="0"
                      :max="editData.max_hp"
                      class="edit-input edit-hp"
                      title="Current HP"
                    /> / <input
                      type="number"
                      v-model.number="editData.max_hp"
                      :min="1"
                      class="edit-input edit-hp"
                      title="Max HP"
                    />
                  </span>
                  <span v-else class="stat-value">{{ data.current_hp }} / {{ data.max_hp }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Hit Dice</span>
                  <span class="stat-value">{{ hitDiceString }}</span>
                </div>
                <div class="combat-stat">
                  <span class="stat-label">Proficiency</span>
                  <span class="stat-value">{{ formatModifier(proficiencyBonus) }}</span>
                </div>
              </div>
            </section>

            <!-- Saving Throws -->
            <section class="sheet-section">
              <h2 class="section-title">Saving Throws</h2>
              <div class="saves-list">
                <div v-for="(score, ability) in data.abilities" :key="ability" class="save-item">
                  <span class="save-proficient" :class="{ active: isProficientSave(ability) }">*</span>
                  <span class="save-name">{{ ability }}</span>
                  <span class="save-bonus">{{ formatModifier(getSaveBonus(ability, score)) }}</span>
                </div>
              </div>
            </section>

            <!-- Attacks -->
            <section class="sheet-section" v-if="hasAttacks">
              <h2 class="section-title">Attacks</h2>
              <div class="attacks-list">
                <div v-for="attack in attacks" :key="attack.name" class="attack-item">
                  <span class="attack-name">{{ attack.name }}</span>
                  <span class="attack-bonus">{{ formatModifier(attack.attackBonus) }}</span>
                  <span class="attack-damage">{{ attack.damage }}</span>
                </div>
              </div>
            </section>
          </div>

          <!-- Middle Column: Skills -->
          <div class="sheet-column">
            <section class="sheet-section">
              <h2 class="section-title">Skills</h2>
              <div class="skills-list">
                <div v-for="skill in allSkills" :key="skill.name" class="skill-item">
                  <span class="skill-proficient" :class="{ active: isProficientSkill(skill.name) }">*</span>
                  <span class="skill-name">{{ skill.name }}</span>
                  <span class="skill-ability">({{ skill.ability.slice(0, 3) }})</span>
                  <span class="skill-bonus">{{ formatModifier(getSkillBonus(skill)) }}</span>
                </div>
              </div>
            </section>
          </div>

          <!-- Right Column: Features, Spells, Equipment -->
          <div class="sheet-column">
            <!-- Proficiencies -->
            <section class="sheet-section">
              <h2 class="section-title">Proficiencies</h2>
              <div class="proficiency-group" v-if="data.proficiencies.armor.length">
                <strong>Armor:</strong> {{ data.proficiencies.armor.join(', ') }}
              </div>
              <div class="proficiency-group" v-if="data.proficiencies.weapons.length">
                <strong>Weapons:</strong> {{ data.proficiencies.weapons.join(', ') }}
              </div>
              <div class="proficiency-group" v-if="data.proficiencies.tools.length">
                <strong>Tools:</strong> {{ data.proficiencies.tools.join(', ') }}
              </div>
              <div class="proficiency-group" v-if="data.proficiencies.languages.length">
                <strong>Languages:</strong> {{ data.proficiencies.languages.join(', ') }}
              </div>
            </section>

            <!-- Class Features -->
            <section class="sheet-section" v-if="data.class_features.length">
              <h2 class="section-title">Features & Traits</h2>
              <div v-if="loadingFeatureDetails" class="loading-indicator">Loading feature details...</div>
              <ul class="feature-list">
                <li v-for="feature in data.class_features" :key="`${feature.name}-${feature.class_name}-${feature.level}`" class="feature-item">
                  <div class="feature-header" @click="toggleFeature(feature)">
                    <strong>{{ feature.name }}</strong>
                    <span class="feature-source">({{ feature.subclass_name || feature.class_name }} Lv{{ feature.level }})</span>
                    <span v-if="getFeatureDescription(feature)" class="expand-icon">{{ isFeatureExpanded(feature) ? '▼' : '▶' }}</span>
                  </div>
                  <div v-if="isFeatureExpanded(feature) && getFeatureDescription(feature)" class="feature-description">
                    {{ getFeatureDescription(feature) }}
                  </div>
                </li>
              </ul>
            </section>

            <!-- Feats -->
            <section class="sheet-section" v-if="data.feats.length">
              <h2 class="section-title">Feats</h2>
              <ul class="feature-list">
                <li v-for="feat in data.feats" :key="feat">{{ feat }}</li>
              </ul>
            </section>

            <!-- Spellcasting Summary (brief) -->
            <section class="sheet-section" v-if="isSpellcaster">
              <h2 class="section-title">Spellcasting</h2>
              <div class="spell-stats">
                <div class="spell-stat">
                  <span class="stat-label">Spell Save DC</span>
                  <span class="stat-value">{{ spellSaveDC }}</span>
                </div>
                <div class="spell-stat">
                  <span class="stat-label">Spell Attack</span>
                  <span class="stat-value">{{ formatModifier(spellAttackBonus) }}</span>
                </div>
              </div>
              <p class="spell-note">See Spells tab for full spell list</p>
            </section>

            <!-- Personality -->
            <section class="sheet-section" v-if="hasPersonality || isEditing">
              <h2 class="section-title">Personality</h2>
              <!-- Edit mode -->
              <template v-if="isEditing && editData">
                <div class="personality-item">
                  <strong>Traits:</strong>
                  <textarea
                    v-model="editData.personality.traits"
                    class="edit-textarea"
                    placeholder="Personality traits..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="personality-item">
                  <strong>Ideals:</strong>
                  <textarea
                    v-model="editData.personality.ideals"
                    class="edit-textarea"
                    placeholder="Ideals..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="personality-item">
                  <strong>Bonds:</strong>
                  <textarea
                    v-model="editData.personality.bonds"
                    class="edit-textarea"
                    placeholder="Bonds..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="personality-item">
                  <strong>Flaws:</strong>
                  <textarea
                    v-model="editData.personality.flaws"
                    class="edit-textarea"
                    placeholder="Flaws..."
                    rows="2"
                  ></textarea>
                </div>
              </template>
              <!-- View mode -->
              <template v-else>
                <div v-if="data.personality.traits" class="personality-item">
                  <strong>Traits:</strong> {{ data.personality.traits }}
                </div>
                <div v-if="data.personality.ideals" class="personality-item">
                  <strong>Ideals:</strong> {{ data.personality.ideals }}
                </div>
                <div v-if="data.personality.bonds" class="personality-item">
                  <strong>Bonds:</strong> {{ data.personality.bonds }}
                </div>
                <div v-if="data.personality.flaws" class="personality-item">
                  <strong>Flaws:</strong> {{ data.personality.flaws }}
                </div>
              </template>
            </section>

            <!-- Legendary Actions (for NPCs) -->
            <section class="sheet-section legendary-section" v-if="isNpc && (hasLegendaryActions || isEditing)">
              <h2 class="section-title">Legendary Actions</h2>

              <!-- Edit mode -->
              <template v-if="isEditing && editData">
                <div class="legendary-count-edit">
                  <label>Actions per Round:</label>
                  <select v-model.number="editData.legendary_action_count" class="form-select-sm">
                    <option :value="1">1</option>
                    <option :value="2">2</option>
                    <option :value="3">3</option>
                    <option :value="4">4</option>
                    <option :value="5">5</option>
                  </select>
                  <button type="button" class="btn-add-legendary" @click="addLegendaryActionEdit">
                    + Add Action
                  </button>
                </div>

                <div v-for="(action, index) in editData.legendary_actions" :key="index" class="legendary-action-edit">
                  <div class="action-edit-header">
                    <input
                      v-model="action.name"
                      type="text"
                      class="edit-input"
                      placeholder="Action name"
                    />
                    <div class="action-cost-edit">
                      <label>Cost:</label>
                      <select v-model.number="action.cost" class="form-select-sm">
                        <option :value="1">1</option>
                        <option :value="2">2</option>
                        <option :value="3">3</option>
                      </select>
                    </div>
                    <button type="button" class="btn-remove-legendary" @click="removeLegendaryActionEdit(index)">
                      x
                    </button>
                  </div>
                  <textarea
                    v-model="action.description"
                    class="edit-textarea action-desc-edit"
                    placeholder="Action description..."
                    rows="2"
                  ></textarea>
                </div>

                <p v-if="editData.legendary_actions.length === 0" class="no-actions-hint">
                  No legendary actions defined. Click "+ Add Action" to add one.
                </p>
              </template>

              <!-- View mode -->
              <template v-else>
                <p class="legendary-intro">
                  {{ data.character_name }} can take {{ data.legendary_action_count || 3 }} legendary actions,
                  choosing from the options below. Only one legendary action can be used at a time and only at
                  the end of another creature's turn.
                </p>
                <div v-for="(action, index) in data.legendary_actions" :key="index" class="legendary-action-view">
                  <strong>{{ action.name }}</strong>
                  <span v-if="action.cost > 1"> (Costs {{ action.cost }} Actions)</span>.
                  {{ action.description }}
                </div>
              </template>
            </section>
          </div>
        </div>

        <!-- Spells Tab Content -->
        <div v-else-if="activeTab === 'spells'" class="spells-sheet">
          <!-- Spellcasting Header -->
          <div class="spells-header">
            <div class="spells-header-actions">
              <button
                v-if="!isSpellEditMode"
                @click="startSpellEdit"
                class="btn-edit-small"
              >
                Edit Spells
              </button>
              <button
                v-else
                @click="cancelSpellEdit"
                class="btn-secondary btn-small"
              >
                Done
              </button>
            </div>
            <div class="spellcasting-info">
              <div class="spell-stat-box">
                <span class="stat-label">Spellcasting Class</span>
                <span class="stat-value">{{ spellcastingClass?.class_name || 'None' }}</span>
              </div>
              <div class="spell-stat-box">
                <span class="stat-label">Spellcasting Ability</span>
                <span class="stat-value">{{ spellcastingAbility.slice(0, 3).toUpperCase() }}</span>
              </div>
              <div class="spell-stat-box">
                <span class="stat-label">Spell Save DC</span>
                <span class="stat-value">{{ spellSaveDC }}</span>
              </div>
              <div class="spell-stat-box">
                <span class="stat-label">Spell Attack Bonus</span>
                <span class="stat-value">{{ formatModifier(spellAttackBonus) }}</span>
              </div>
              <div v-if="maxSpellsKnown !== null" class="spell-stat-box">
                <span class="stat-label">Spells Known</span>
                <span class="stat-value" :class="{ 'at-limit': isAtSpellLimit }">
                  {{ currentSpellsCount }}/{{ maxSpellsKnown }}
                </span>
              </div>
            </div>

            <!-- Spell Slots -->
            <div v-if="Object.keys(calculatedSpellSlots).length" class="spell-slots-row">
              <div v-for="level in Object.keys(calculatedSpellSlots).map(Number).sort((a, b) => a - b)" :key="level" class="spell-slot-box">
                <span class="slot-level-label">{{ level }}</span>
                <div class="slot-circles">
                  <span
                    v-for="i in calculatedSpellSlots[level].max"
                    :key="i"
                    class="slot-circle"
                    :class="{ used: i > calculatedSpellSlots[level].current }"
                  ></span>
                </div>
              </div>
            </div>
          </div>

          <!-- Spell Lists by Level -->
          <div class="spell-levels-grid">
            <!-- Cantrips -->
            <div class="spell-level-section">
              <div class="level-header-box">
                <span class="level-title">Cantrips</span>
                <span v-if="isSpellEditMode && isFullListCaster" class="edit-hint">Click to toggle</span>
                <button
                  v-if="isSpellEditMode && !isFullListCaster"
                  @click="openSpellPicker(0)"
                  class="btn-add-spell"
                  :disabled="isAtCantripLimit"
                >
                  + Add
                </button>
                <div v-if="maxCantripsKnown > 0" class="level-slots-info">
                  <span class="slots-total" :class="{ 'at-limit': isAtCantripLimit }">
                    {{ currentCantripsCount }}/{{ maxCantripsKnown }} known
                  </span>
                  <div class="slots-circles-header">
                    <span
                      v-for="i in maxCantripsKnown"
                      :key="i"
                      class="slot-circle-sm"
                      :class="{ used: i > currentCantripsCount }"
                    ></span>
                  </div>
                </div>
              </div>
              <div class="spell-list">
                <div
                  v-for="spellName in getSortedSpellsForLevel(0)"
                  :key="spellName"
                  class="spell-item"
                  :class="{ expanded: expandedSpells.has(spellName), 'spell-in-book': isSpellEditMode && isSpellKnown(spellName) }"
                >
                  <div class="spell-row" @click="isSpellEditMode ? toggleSpellInBook(spellName, true) : toggleSpellExpansion(spellName)">
                    <span v-if="isSpellEditMode" class="spell-checkbox" :class="{ checked: isSpellKnown(spellName) }">
                      {{ isSpellKnown(spellName) ? '✓' : '' }}
                    </span>
                    <span class="spell-name">{{ spellName }}</span>
                    <span v-if="!isSpellEditMode" class="expand-icon">{{ expandedSpells.has(spellName) ? '-' : '+' }}</span>
                  </div>
                  <div v-if="expandedSpells.has(spellName)" class="spell-details">
                    <div v-if="loadingSpellDetails.has(spellName)" class="loading-details">
                      Loading...
                    </div>
                    <template v-else-if="spellDetails[spellName]">
                      <div class="spell-meta">
                        <span>{{ spellDetails[spellName].school }} cantrip</span>
                      </div>
                      <div class="spell-properties">
                        <div><strong>Casting Time:</strong> {{ spellDetails[spellName].time[0]?.number }} {{ spellDetails[spellName].time[0]?.unit }}</div>
                        <div><strong>Range:</strong> {{ spellDetails[spellName].range.distance?.amount || '' }} {{ spellDetails[spellName].range.distance?.type || spellDetails[spellName].range.type }}</div>
                        <div><strong>Components:</strong>
                          {{ spellDetails[spellName].components.v ? 'V' : '' }}{{ spellDetails[spellName].components.s ? 'S' : '' }}{{ spellDetails[spellName].components.m ? 'M' : '' }}
                        </div>
                        <div><strong>Duration:</strong> {{ spellDetails[spellName].duration[0]?.concentration ? 'Concentration, ' : '' }}{{ spellDetails[spellName].duration[0]?.duration?.amount || '' }} {{ spellDetails[spellName].duration[0]?.duration?.type || spellDetails[spellName].duration[0]?.type }}</div>
                      </div>
                      <div class="spell-description">
                        <p v-for="(entry, idx) in spellDetails[spellName].entries" :key="idx">
                          {{ formatSpellEntry(entry) }}
                        </p>
                      </div>
                    </template>
                  </div>
                </div>
                <div v-if="!getSortedSpellsForLevel(0).length" class="no-spells-message">
                  No cantrips
                </div>
              </div>
            </div>

            <!-- Spell Levels 1-9 -->
            <template v-for="level in [1, 2, 3, 4, 5, 6, 7, 8, 9]" :key="level">
              <div v-if="level <= maxSpellLevel" class="spell-level-section">
                <div class="level-header-box">
                  <span class="level-title">Level {{ level }}</span>
                  <span v-if="isSpellEditMode && isFullListCaster" class="edit-hint">Spellbook (no limit)</span>
                  <span v-if="isSpellEditMode && !isFullListCaster && maxSpellsKnown !== null" class="spell-count" :class="{ 'at-limit': isAtSpellLimit }">
                    {{ currentSpellsCount }}/{{ maxSpellsKnown }} known
                  </span>
                  <button
                    v-if="isSpellEditMode && !isFullListCaster"
                    @click="openSpellPicker(level)"
                    class="btn-add-spell"
                    :disabled="isAtSpellLimit"
                  >
                    + Add
                  </button>
                  <div class="level-slots-info">
                    <span v-if="calculatedSpellSlots[level]" class="slots-total">
                      {{ calculatedSpellSlots[level].max }} slots
                    </span>
                    <span v-else class="slots-total">0 slots</span>
                    <div v-if="calculatedSpellSlots[level]" class="slots-circles-header">
                      <span
                        v-for="i in calculatedSpellSlots[level].max"
                        :key="i"
                        class="slot-circle-sm"
                        :class="{ used: i > calculatedSpellSlots[level].current }"
                      ></span>
                    </div>
                  </div>
                </div>
                <div class="spell-list">
                  <div
                    v-for="spellName in getSortedSpellsForLevel(level)"
                    :key="spellName"
                    class="spell-item"
                    :class="{ expanded: expandedSpells.has(spellName), 'spell-in-book': isSpellEditMode && isSpellKnown(spellName) }"
                  >
                    <div class="spell-row" @click="isSpellEditMode ? toggleSpellInBook(spellName, false) : toggleSpellExpansion(spellName)">
                      <span v-if="isSpellEditMode" class="spell-checkbox" :class="{ checked: isSpellKnown(spellName) }">
                        {{ isSpellKnown(spellName) ? '✓' : '' }}
                      </span>
                      <span class="spell-name">{{ spellName }}</span>
                      <span v-if="!isSpellEditMode" class="expand-icon">{{ expandedSpells.has(spellName) ? '-' : '+' }}</span>
                    </div>
                    <div v-if="expandedSpells.has(spellName)" class="spell-details">
                      <div v-if="loadingSpellDetails.has(spellName)" class="loading-details">
                        Loading...
                      </div>
                      <template v-else-if="spellDetails[spellName]">
                        <div class="spell-meta">
                          <span>{{ spellDetails[spellName].level === 0 ? 'Cantrip' : `Level ${spellDetails[spellName].level}` }} {{ spellDetails[spellName].school }}</span>
                          <span v-if="spellDetails[spellName].duration[0]?.concentration" class="concentration-tag">Concentration</span>
                        </div>
                        <div class="spell-properties">
                          <div><strong>Casting Time:</strong> {{ spellDetails[spellName].time[0]?.number }} {{ spellDetails[spellName].time[0]?.unit }}</div>
                          <div><strong>Range:</strong> {{ spellDetails[spellName].range.distance?.amount || '' }} {{ spellDetails[spellName].range.distance?.type || spellDetails[spellName].range.type }}</div>
                          <div><strong>Components:</strong>
                            {{ spellDetails[spellName].components.v ? 'V' : '' }}{{ spellDetails[spellName].components.s ? 'S' : '' }}{{ spellDetails[spellName].components.m ? 'M' : '' }}
                          </div>
                          <div><strong>Duration:</strong> {{ spellDetails[spellName].duration[0]?.concentration ? 'Concentration, ' : '' }}{{ spellDetails[spellName].duration[0]?.duration?.amount || '' }} {{ spellDetails[spellName].duration[0]?.duration?.type || spellDetails[spellName].duration[0]?.type }}</div>
                        </div>
                        <div class="spell-description">
                          <p v-for="(entry, idx) in spellDetails[spellName].entries" :key="idx">
                            {{ formatSpellEntry(entry) }}
                          </p>
                        </div>
                      </template>
                    </div>
                  </div>
                  <div v-if="!getSortedSpellsForLevel(level).length" class="no-spells-message">
                    -
                  </div>
                </div>
              </div>
            </template>
          </div>

          <!-- Spell Picker Modal -->
          <div v-if="showSpellPicker" class="spell-picker-overlay" @click.self="showSpellPicker = false">
            <div class="spell-picker-modal">
              <div class="spell-picker-header">
                <h3>Add {{ spellPickerLevel === 0 ? 'Cantrip' : `Level ${spellPickerLevel} Spell` }}</h3>
                <button @click="showSpellPicker = false" class="btn-close">×</button>
              </div>
              <div class="spell-picker-content">
                <div v-if="spellsForPicker.length === 0" class="no-spells-available">
                  No spells available for this level
                </div>
                <div v-else class="spell-picker-list">
                  <button
                    v-for="spell in spellsForPicker"
                    :key="spell.name"
                    @click="!isSpellKnown(spell.name) && addSpell(spell)"
                    class="spell-picker-item"
                    :class="{ 'spell-known': isSpellKnown(spell.name) }"
                    :disabled="isSpellKnown(spell.name)"
                  >
                    <span class="picker-spell-name">{{ spell.name }}</span>
                    <span v-if="isSpellKnown(spell.name)" class="picker-spell-known-badge">Known</span>
                    <span v-else class="picker-spell-school">{{ spell.school }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Equipment Tab Content -->
        <div v-else-if="activeTab === 'equipment'" class="equipment-sheet">
          <div class="equipment-content">
            <!-- Currency Section -->
            <section class="equipment-section currency-section">
              <div class="section-header">
                <h2 class="section-title">Currency</h2>
                <button @click="showCurrencyEditor = !showCurrencyEditor" class="btn-edit-small">
                  {{ showCurrencyEditor ? 'Done' : 'Edit' }}
                </button>
              </div>
              <div v-if="!showCurrencyEditor" class="currency-display">
                <div class="currency-item large">
                  <span class="currency-icon">PP</span>
                  <span class="currency-value">{{ data.currency.platinum }}</span>
                </div>
                <div class="currency-item large">
                  <span class="currency-icon gold">GP</span>
                  <span class="currency-value">{{ data.currency.gold }}</span>
                </div>
                <div class="currency-item">
                  <span class="currency-icon silver">EP</span>
                  <span class="currency-value">{{ data.currency.electrum }}</span>
                </div>
                <div class="currency-item">
                  <span class="currency-icon silver">SP</span>
                  <span class="currency-value">{{ data.currency.silver }}</span>
                </div>
                <div class="currency-item">
                  <span class="currency-icon copper">CP</span>
                  <span class="currency-value">{{ data.currency.copper }}</span>
                </div>
              </div>
              <div v-else class="currency-editor">
                <div class="currency-input-row">
                  <label>PP</label>
                  <input type="number" v-model.number="currencyEdit.platinum" min="0" />
                </div>
                <div class="currency-input-row">
                  <label>GP</label>
                  <input type="number" v-model.number="currencyEdit.gold" min="0" />
                </div>
                <div class="currency-input-row">
                  <label>EP</label>
                  <input type="number" v-model.number="currencyEdit.electrum" min="0" />
                </div>
                <div class="currency-input-row">
                  <label>SP</label>
                  <input type="number" v-model.number="currencyEdit.silver" min="0" />
                </div>
                <div class="currency-input-row">
                  <label>CP</label>
                  <input type="number" v-model.number="currencyEdit.copper" min="0" />
                </div>
                <button @click="saveCurrency" class="btn-primary btn-small">Save Currency</button>
              </div>
            </section>

            <!-- Equipped Items Section -->
            <section class="equipment-section">
              <div class="section-header">
                <h2 class="section-title">Equipped</h2>
              </div>
              <div class="equipped-slots">
                <div class="equip-slot">
                  <span class="slot-label">Armor</span>
                  <select v-model="equippedItems.armor" @change="updateEquipped">
                    <option :value="null">None</option>
                    <option v-for="item in armorItems" :key="item.name" :value="item.name">
                      {{ item.name }}
                    </option>
                  </select>
                </div>
                <div class="equip-slot">
                  <span class="slot-label">Shield</span>
                  <select v-model="equippedItems.shield" @change="updateEquipped">
                    <option :value="null">None</option>
                    <option v-for="item in shieldItems" :key="item.name" :value="item.name">
                      {{ item.name }}
                    </option>
                  </select>
                </div>
                <div class="equip-slot">
                  <span class="slot-label">Main Hand</span>
                  <select v-model="equippedItems.main_hand" @change="updateEquipped">
                    <option :value="null">None</option>
                    <option v-for="item in weaponItems" :key="item.name" :value="item.name">
                      {{ item.name }}
                    </option>
                  </select>
                </div>
                <div class="equip-slot">
                  <span class="slot-label">Off Hand</span>
                  <select v-model="equippedItems.off_hand" @change="updateEquipped">
                    <option :value="null">None</option>
                    <option v-for="item in offHandItems" :key="item.name" :value="item.name">
                      {{ item.name }}
                    </option>
                  </select>
                </div>
              </div>
            </section>

            <!-- Inventory Section -->
            <section class="equipment-section inventory-section">
              <div class="section-header">
                <h2 class="section-title">Inventory</h2>
                <button @click="showAddItem = true" class="btn-add">+ Add Item</button>
              </div>

              <!-- Add Item Form -->
              <div v-if="showAddItem" class="add-item-form">
                <div class="form-row">
                  <select v-model="itemSourceFilter" class="input-source-filter" @change="searchItems(itemSearchQuery)">
                    <option value="all">All Sources</option>
                    <option v-for="source in availableItemSources" :key="source" :value="source">
                      {{ source }}
                    </option>
                  </select>
                  <div class="search-input-container">
                    <input
                      :value="itemSearchQuery"
                      @input="onItemSearchInput"
                      @blur="closeItemDropdown"
                      @focus="itemSearchQuery.length >= 2 && (showItemDropdown = itemSearchResults.length > 0)"
                      placeholder="Search items..."
                      class="input-name"
                      autocomplete="off"
                    />
                    <span v-if="isSearchingItems" class="search-spinner">...</span>
                    <!-- Search Results Dropdown -->
                    <div v-if="showItemDropdown" class="search-dropdown">
                      <div
                        v-for="item in itemSearchResults"
                        :key="`${item.name}-${item.source}`"
                        class="search-result-item"
                        @mousedown.prevent="selectSearchItem(item)"
                      >
                        <span class="result-name">{{ item.name }}</span>
                        <span class="result-meta">
                          <span class="result-type">{{ item.typeName }}</span>
                          <span v-if="item.rarity && item.rarity !== 'none'" class="result-rarity">{{ item.rarity }}</span>
                          <span class="result-source">{{ item.source }}</span>
                        </span>
                      </div>
                      <div v-if="itemSearchResults.length === 0" class="no-results">
                        No items found
                      </div>
                    </div>
                  </div>
                </div>
                <div class="form-row">
                  <input
                    v-model.number="newItem.quantity"
                    type="number"
                    min="1"
                    placeholder="Qty"
                    class="input-qty"
                  />
                  <input
                    v-model="newItem.notes"
                    placeholder="Notes (optional)"
                    class="input-notes"
                  />
                </div>
                <div class="form-actions">
                  <button @click="addItem" class="btn-primary btn-small" :disabled="!newItem.name">Add</button>
                  <button @click="cancelAddItem" class="btn-secondary btn-small">Cancel</button>
                </div>
              </div>

              <!-- Inventory List -->
              <EmptyState
                v-if="data.inventory.length === 0 && !showAddItem"
                variant="generic"
                title="No items in inventory"
                description="Click '+ Add Item' to add some"
              />

              <div v-else class="inventory-list">
                <div
                  v-for="item in data.inventory"
                  :key="item.name"
                  class="inventory-item"
                  :class="{ expanded: expandedItems.has(`${item.name}:${item.source || 'PHB'}`) }"
                >
                  <div class="item-header" @click="toggleItemExpansion(item.name, item.source)">
                    <div class="item-main">
                      <span class="item-name">{{ item.name }}</span>
                      <span class="item-qty">x{{ item.quantity }}</span>
                    </div>
                    <div class="item-actions">
                      <button @click.stop="decrementItem(item)" class="btn-qty" title="Remove one">-</button>
                      <button @click.stop="incrementItem(item)" class="btn-qty" title="Add one">+</button>
                      <button @click.stop="removeItemCompletely(item)" class="btn-remove" title="Remove all">x</button>
                    </div>
                  </div>

                  <div v-if="expandedItems.has(`${item.name}:${item.source || 'PHB'}`)" class="item-details">
                    <div v-if="item.notes" class="item-notes">
                      <strong>Notes:</strong> {{ item.notes }}
                    </div>
                    <div class="item-meta-row">
                      <span v-if="item.weight"><strong>Weight:</strong> {{ item.weight }} lb</span>
                      <span v-if="item.source"><strong>Source:</strong> {{ item.source }}</span>
                    </div>

                    <div v-if="loadingItemDetails.has(`${item.name}:${item.source || 'PHB'}`)" class="loading-details">
                      Loading catalog details...
                    </div>
                    <template v-else-if="itemDetails[`${item.name}:${item.source || 'PHB'}`]">
                      <div class="catalog-details">
                        <div class="item-type-rarity">
                          <span v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].type" class="item-type">
                            {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].type }}
                          </span>
                          <span v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].rarity" class="item-rarity">
                            {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].rarity }}
                          </span>
                        </div>
                        <div class="item-stats">
                          <span v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].ac">
                            <strong>AC:</strong> {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].ac }}
                          </span>
                          <span v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].dmg1">
                            <strong>Damage:</strong> {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].dmg1 }} {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].dmgType || '' }}
                          </span>
                          <span v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].range">
                            <strong>Range:</strong> {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].range }}
                          </span>
                        </div>
                        <div v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].property?.length" class="item-properties">
                          <strong>Properties:</strong> {{ itemDetails[`${item.name}:${item.source || 'PHB'}`].property?.join(', ') }}
                        </div>
                        <div v-if="itemDetails[`${item.name}:${item.source || 'PHB'}`].entries?.length" class="item-description">
                          <p v-for="(entry, idx) in itemDetails[`${item.name}:${item.source || 'PHB'}`].entries" :key="idx">
                            {{ formatItemEntry(entry) }}
                          </p>
                        </div>
                      </div>
                    </template>
                  </div>
                </div>
              </div>

              <!-- Total Weight -->
              <div class="inventory-footer">
                <span class="total-weight">
                  <strong>Total Weight:</strong> {{ totalWeight.toFixed(1) }} lb
                </span>
              </div>
            </section>
          </div>
        </div>

        <!-- Details Tab Content -->
        <div v-else-if="activeTab === 'details'" class="details-sheet">
          <div class="details-content">
            <!-- Player Info Section -->
            <section class="details-section">
              <h2 class="section-title">Player Info</h2>
              <div class="player-info-row">
                <template v-if="isEditing && editData">
                  <label class="field-label">Player:</label>
                  <select
                    v-model="editData.player_id"
                    class="edit-select"
                  >
                    <option :value="null">— No Player —</option>
                    <option
                      v-for="player in playerStore.players"
                      :key="player.id"
                      :value="player.id"
                    >
                      {{ player.name }}
                    </option>
                  </select>
                </template>
                <template v-else>
                  <span class="field-label">Player:</span>
                  <span class="field-value">{{ currentPlayerName }}</span>
                </template>
              </div>
            </section>

            <!-- Appearance Section -->
            <section class="details-section">
              <h2 class="section-title">Appearance</h2>
              <!-- Edit mode -->
              <template v-if="isEditing && editData">
                <div class="appearance-grid">
                  <div class="appearance-field">
                    <label>Age</label>
                    <input v-model="editData.appearance.age" class="edit-input" placeholder="Age..." />
                  </div>
                  <div class="appearance-field">
                    <label>Height</label>
                    <input v-model="editData.appearance.height" class="edit-input" placeholder="Height..." />
                  </div>
                  <div class="appearance-field">
                    <label>Weight</label>
                    <input v-model="editData.appearance.weight" class="edit-input" placeholder="Weight..." />
                  </div>
                  <div class="appearance-field">
                    <label>Eyes</label>
                    <input v-model="editData.appearance.eyes" class="edit-input" placeholder="Eye color..." />
                  </div>
                  <div class="appearance-field">
                    <label>Hair</label>
                    <input v-model="editData.appearance.hair" class="edit-input" placeholder="Hair color/style..." />
                  </div>
                  <div class="appearance-field">
                    <label>Skin</label>
                    <input v-model="editData.appearance.skin" class="edit-input" placeholder="Skin tone..." />
                  </div>
                </div>
                <div class="appearance-textarea-section">
                  <label>Physical Description</label>
                  <textarea
                    v-model="editData.appearance.physical_description"
                    class="edit-textarea"
                    placeholder="Describe your character's physical appearance..."
                    rows="3"
                  ></textarea>
                </div>
                <div class="appearance-textarea-section">
                  <label>Distinctive Features</label>
                  <textarea
                    v-model="editData.appearance.distinctive_features"
                    class="edit-textarea"
                    placeholder="Scars, tattoos, birthmarks, unusual features..."
                    rows="2"
                  ></textarea>
                </div>
              </template>
              <!-- View mode -->
              <template v-else>
                <div class="appearance-grid-view">
                  <div v-if="data.appearance?.age" class="appearance-item">
                    <strong>Age:</strong> {{ data.appearance.age }}
                  </div>
                  <div v-if="data.appearance?.height" class="appearance-item">
                    <strong>Height:</strong> {{ data.appearance.height }}
                  </div>
                  <div v-if="data.appearance?.weight" class="appearance-item">
                    <strong>Weight:</strong> {{ data.appearance.weight }}
                  </div>
                  <div v-if="data.appearance?.eyes" class="appearance-item">
                    <strong>Eyes:</strong> {{ data.appearance.eyes }}
                  </div>
                  <div v-if="data.appearance?.hair" class="appearance-item">
                    <strong>Hair:</strong> {{ data.appearance.hair }}
                  </div>
                  <div v-if="data.appearance?.skin" class="appearance-item">
                    <strong>Skin:</strong> {{ data.appearance.skin }}
                  </div>
                </div>
                <div v-if="data.appearance?.physical_description" class="appearance-text">
                  <strong>Physical Description:</strong>
                  <p>{{ data.appearance.physical_description }}</p>
                </div>
                <div v-if="data.appearance?.distinctive_features" class="appearance-text">
                  <strong>Distinctive Features:</strong>
                  <p>{{ data.appearance.distinctive_features }}</p>
                </div>
                <EmptyState
                  v-if="!hasAppearanceData"
                  variant="generic"
                  title="No appearance details"
                  description="Click 'Edit' to add"
                />
              </template>
            </section>

            <!-- Background Section -->
            <section class="details-section">
              <h2 class="section-title">Background</h2>
              <!-- Edit mode -->
              <template v-if="isEditing && editData">
                <div class="background-field">
                  <label>Background Feature</label>
                  <textarea
                    v-model="editData.background_feature"
                    class="edit-textarea"
                    placeholder="Your background feature..."
                    rows="3"
                  ></textarea>
                </div>
                <div class="background-field">
                  <label>Backstory</label>
                  <textarea
                    v-model="editData.backstory"
                    class="edit-textarea"
                    placeholder="Your character's history and origins..."
                    rows="8"
                  ></textarea>
                </div>
              </template>
              <!-- View mode -->
              <template v-else>
                <div v-if="data.background_feature" class="background-text">
                  <strong>Background Feature:</strong>
                  <p>{{ data.background_feature }}</p>
                </div>
                <div v-if="data.backstory" class="background-text">
                  <strong>Backstory:</strong>
                  <p class="backstory-text">{{ data.backstory }}</p>
                </div>
                <EmptyState
                  v-if="!data.background_feature && !data.backstory"
                  variant="generic"
                  title="No background details"
                  description="Click 'Edit' to add"
                />
              </template>
            </section>

            <!-- Roleplay Notes Section -->
            <section class="details-section">
              <h2 class="section-title">Roleplay Notes</h2>
              <!-- Edit mode -->
              <template v-if="isEditing && editData">
                <div class="roleplay-field">
                  <label>Voice & Mannerisms</label>
                  <textarea
                    v-model="editData.roleplay_notes.voice_and_mannerisms"
                    class="edit-textarea"
                    placeholder="How does your character speak? Accent, catchphrases, gestures..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="roleplay-field">
                  <label>Key Relationships</label>
                  <textarea
                    v-model="editData.roleplay_notes.key_relationships"
                    class="edit-textarea"
                    placeholder="Important NPCs, family, rivals, mentors..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="roleplay-field">
                  <label>Character Goals</label>
                  <textarea
                    v-model="editData.roleplay_notes.character_goals"
                    class="edit-textarea"
                    placeholder="What does your character want to achieve?"
                    rows="2"
                  ></textarea>
                </div>
                <div class="roleplay-field">
                  <label>Play Reminders</label>
                  <textarea
                    v-model="editData.roleplay_notes.play_reminders"
                    class="edit-textarea"
                    placeholder="Notes to yourself about how to play this character..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="roleplay-field">
                  <label>Allies & Organizations</label>
                  <textarea
                    v-model="editData.roleplay_notes.allies_and_organizations"
                    class="edit-textarea"
                    placeholder="Factions, guilds, orders your character belongs to or is allied with..."
                    rows="2"
                  ></textarea>
                </div>
                <div class="roleplay-field">
                  <label>Additional Treasure Notes</label>
                  <textarea
                    v-model="editData.roleplay_notes.additional_treasure_notes"
                    class="edit-textarea"
                    placeholder="Notes about special items, hidden stashes, debts owed..."
                    rows="2"
                  ></textarea>
                </div>
              </template>
              <!-- View mode -->
              <template v-else>
                <div v-if="data.roleplay_notes?.voice_and_mannerisms" class="roleplay-item">
                  <strong>Voice & Mannerisms:</strong>
                  <p>{{ data.roleplay_notes.voice_and_mannerisms }}</p>
                </div>
                <div v-if="data.roleplay_notes?.key_relationships" class="roleplay-item">
                  <strong>Key Relationships:</strong>
                  <p>{{ data.roleplay_notes.key_relationships }}</p>
                </div>
                <div v-if="data.roleplay_notes?.character_goals" class="roleplay-item">
                  <strong>Character Goals:</strong>
                  <p>{{ data.roleplay_notes.character_goals }}</p>
                </div>
                <div v-if="data.roleplay_notes?.play_reminders" class="roleplay-item">
                  <strong>Play Reminders:</strong>
                  <p>{{ data.roleplay_notes.play_reminders }}</p>
                </div>
                <div v-if="data.roleplay_notes?.allies_and_organizations" class="roleplay-item">
                  <strong>Allies & Organizations:</strong>
                  <p>{{ data.roleplay_notes.allies_and_organizations }}</p>
                </div>
                <div v-if="data.roleplay_notes?.additional_treasure_notes" class="roleplay-item">
                  <strong>Additional Treasure Notes:</strong>
                  <p>{{ data.roleplay_notes.additional_treasure_notes }}</p>
                </div>
                <EmptyState
                  v-if="!hasRoleplayNotes"
                  variant="generic"
                  title="No roleplay notes"
                  description="Click 'Edit' to add"
                />
              </template>
            </section>
          </div>
        </div>
      </template>
    </div>

    <!-- Level Up Dialog -->
    <LevelUpDialog
      v-if="data"
      :visible="showLevelUpDialog"
      :character-id="characterId"
      :character-data="data"
      @close="showLevelUpDialog = false"
      @completed="handleLevelUpCompleted"
    />

    <!-- Inventory Manager -->
    <InventoryManager
      v-if="data"
      :visible="showInventoryManager"
      :character-id="characterId"
      :character-data="data"
      @close="showInventoryManager = false"
      @updated="handleInventoryUpdated"
    />

    <!-- PDF Preview Modal -->
    <PdfPreviewModal
      ref="pdfPreviewRef"
      :visible="showPdfPreview"
      :title="pdfPreviewTitle"
      :default-file-name="pdfFileName"
      @close="showPdfPreview = false"
      @retry="printCharacter"
    />

    <!-- Character Print Dialog -->
    <CharacterPrintDialog
      v-if="data"
      :visible="showPrintDialog"
      :character-id="characterId"
      :character-name="data.character_name"
      @close="showPrintDialog = false"
    />
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import LevelUpDialog from '../components/LevelUpDialog.vue'
import InventoryManager from '../components/InventoryManager.vue'
import { PdfPreviewModal, CharacterPrintDialog } from '../../../components/print'
import { PrintService } from '../../../services/PrintService'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import { useCharacterStore } from '../../../stores/characters'
import { usePlayerStore } from '../../../stores/players'
import type { CharacterData, FeatureDetail, FeatureReference } from '../../../types/character'

// Spell summary from catalog
interface SpellSummary {
  name: string
  source: string
  level: number
  school: string
  concentration: boolean
  ritual: boolean
  casting_time: string
  range: string
  components: string
  classes: string[]
  description: string
}

// Full spell details
interface Spell {
  name: string
  source: string
  level: number
  school: string
  time: Array<{ number: number; unit: string }>
  range: { type: string; distance?: { type: string; amount?: number } }
  components: { v?: boolean; s?: boolean; m?: string | { text: string } }
  duration: Array<{ type: string; duration?: { type: string; amount?: number }; concentration?: boolean }>
  entries: Array<string | object>
}

// Full item details from catalog
interface ItemDetails {
  name: string
  source: string
  type?: string
  rarity?: string
  weight?: number
  value?: number
  ac?: number
  dmg1?: string
  dmgType?: string
  property?: string[]
  range?: string
  entries?: Array<string | object>
}

// Class details from catalog (for spell progression)
// Note: Property names are camelCase to match JSON response from Rust serde
interface CatalogClass {
  name: string
  source: string
  cantripProgression?: number[]
  spellsKnownProgression?: number[]
  casterProgression?: string
  spellcastingAbility?: string
}

const route = useRoute()
const router = useRouter()
const characterStore = useCharacterStore()
const playerStore = usePlayerStore()

// Tab navigation
const activeTab = ref<'character' | 'spells' | 'equipment' | 'details'>('character')

// Spell details cache and loading state
const spellDetails = ref<Record<string, Spell>>({})
const loadingSpellDetails = ref<Set<string>>(new Set())
const expandedSpells = ref<Set<string>>(new Set())

// Item details cache and loading state
const itemDetails = ref<Record<string, ItemDetails>>({})
const loadingItemDetails = ref<Set<string>>(new Set())
const expandedItems = ref<Set<string>>(new Set())

// Feature details cache and loading state
const featureDetails = ref<Record<string, FeatureDetail>>({})
const loadingFeatureDetails = ref(false)
const expandedFeatures = ref<Set<string>>(new Set())

// Edit mode
const isEditing = ref(false)
const editData = ref<{
  character_name: string
  alignment: string
  max_hp: number
  current_hp: number
  abilities: {
    strength: number
    dexterity: number
    constitution: number
    intelligence: number
    wisdom: number
    charisma: number
  }
  personality: {
    traits: string
    ideals: string
    bonds: string
    flaws: string
  }
  legendary_actions: Array<{
    name: string
    cost: number
    description: string
  }>
  legendary_action_count: number
  // Extended character details
  player_id: number | null
  appearance: {
    age: string
    height: string
    weight: string
    eyes: string
    hair: string
    skin: string
    physical_description: string
    distinctive_features: string
  }
  backstory: string
  background_feature: string
  roleplay_notes: {
    voice_and_mannerisms: string
    key_relationships: string
    character_goals: string
    play_reminders: string
    allies_and_organizations: string
    additional_treasure_notes: string
  }
} | null>(null)

// Catalog class details (for spell progression from rules)
const catalogClassDetails = ref<CatalogClass | null>(null)

const characterId = computed(() => Number(route.params.id))
const character = computed(() => characterStore.currentCharacter)
const data = computed(() => character.value?.data as CharacterData)

onMounted(async () => {
  await characterStore.getCharacter(characterId.value)
  // Fetch players for the player selector
  await playerStore.fetchPlayers()
})

// Fetch feature details when character data is available
const fetchFeatureDetails = async () => {
  if (!data.value?.class_features?.length) return

  loadingFeatureDetails.value = true
  try {
    const features = data.value.class_features.map((f: FeatureReference) => ({
      name: f.name,
      class_name: f.class_name,
      subclass_name: f.subclass_name,
      source: f.source,
      level: f.level
    }))

    const details = await invoke<FeatureDetail[]>('get_feature_details', { features })

    // Store in cache keyed by feature name
    for (const detail of details) {
      const key = `${detail.name}-${detail.class_name}-${detail.level}`
      featureDetails.value[key] = detail
    }
  } catch (error) {
    console.error('Failed to fetch feature details:', error)
  } finally {
    loadingFeatureDetails.value = false
  }
}

// Watch for character data changes to fetch feature details
watch(() => data.value?.class_features, (newFeatures) => {
  if (newFeatures?.length) {
    fetchFeatureDetails()
  }
}, { immediate: true })

// Fetch spellcasting class details from catalog (for progression data)
const fetchSpellcastingClassDetails = async () => {
  if (!data.value?.classes?.length) return

  // Find the first spellcasting class
  const spellcasters = ['wizard', 'cleric', 'druid', 'bard', 'sorcerer', 'warlock', 'paladin', 'ranger']
  const spellcastingCls = data.value.classes.find(c =>
    spellcasters.includes(c.class_name.toLowerCase())
  )

  if (!spellcastingCls) {
    catalogClassDetails.value = null
    return
  }

  try {
    const classData = await invoke<CatalogClass | null>('get_class_details', {
      className: spellcastingCls.class_name,
      classSource: 'PHB' // Most base classes are PHB
    })
    catalogClassDetails.value = classData
  } catch (error) {
    console.error('Failed to fetch class details:', error)
    catalogClassDetails.value = null
  }
}

// Watch for character data changes to fetch class details
watch(() => data.value?.classes, (newClasses) => {
  if (newClasses?.length) {
    fetchSpellcastingClassDetails()
  }
}, { immediate: true })

// Get feature description from cache
const getFeatureDescription = (feature: FeatureReference): string | null => {
  const key = `${feature.name}-${feature.class_name}-${feature.level}`
  return featureDetails.value[key]?.description || null
}

// Toggle feature expansion
const toggleFeature = (feature: FeatureReference) => {
  const key = `${feature.name}-${feature.class_name}-${feature.level}`
  if (expandedFeatures.value.has(key)) {
    expandedFeatures.value.delete(key)
  } else {
    expandedFeatures.value.add(key)
  }
}

// Check if feature is expanded
const isFeatureExpanded = (feature: FeatureReference): boolean => {
  const key = `${feature.name}-${feature.class_name}-${feature.level}`
  return expandedFeatures.value.has(key)
}

// Ability score helpers
const getModifier = (score: number): number => Math.floor((score - 10) / 2)
const formatModifier = (mod: number): string => mod >= 0 ? `+${mod}` : `${mod}`

// Proficiency bonus based on level
const proficiencyBonus = computed(() => {
  if (!data.value) return 2
  return Math.ceil(data.value.level / 4) + 1
})

// Armor AC calculation
const getArmorAC = (armorName: string, dexMod: number): number => {
  const name = armorName.toLowerCase()

  // Extract magic bonus (+1, +2, +3)
  const magicMatch = name.match(/\+(\d)/)
  const magicBonus = magicMatch ? parseInt(magicMatch[1]) : 0

  // Light armor (full DEX)
  if (name.includes('padded') || (name.includes('leather') && !name.includes('studded'))) {
    return 11 + dexMod + magicBonus
  }
  if (name.includes('studded leather')) {
    return 12 + dexMod + magicBonus
  }

  // Medium armor (DEX max +2)
  const cappedDex = Math.min(dexMod, 2)
  if (name.includes('hide')) {
    return 12 + cappedDex + magicBonus
  }
  if (name.includes('chain shirt')) {
    return 13 + cappedDex + magicBonus
  }
  if (name.includes('scale mail') || name.includes('scale')) {
    return 14 + cappedDex + magicBonus
  }
  if (name.includes('breastplate')) {
    return 14 + cappedDex + magicBonus
  }
  if (name.includes('half plate')) {
    return 15 + cappedDex + magicBonus
  }

  // Heavy armor (no DEX)
  if (name.includes('ring mail')) {
    return 14 + magicBonus
  }
  if (name.includes('chain mail')) {
    return 16 + magicBonus
  }
  if (name.includes('splint')) {
    return 17 + magicBonus
  }
  if (name.includes('plate')) {
    return 18 + magicBonus
  }

  // Default: treat as light armor
  return 11 + dexMod + magicBonus
}

// Base AC calculation
const baseAC = computed(() => {
  if (!data.value) return 10
  const dexMod = getModifier(data.value.abilities.dexterity)

  // If armor is equipped, calculate based on armor type
  if (data.value.equipped.armor) {
    return getArmorAC(data.value.equipped.armor, dexMod)
  }

  // Unarmored: 10 + DEX
  return 10 + dexMod
})

// Check if character has a shield equipped
const hasShield = computed(() => {
  if (!data.value) return false
  return !!data.value.equipped.shield
})

// Skills with their associated abilities
const allSkills = [
  { name: 'Acrobatics', ability: 'dexterity' },
  { name: 'Animal Handling', ability: 'wisdom' },
  { name: 'Arcana', ability: 'intelligence' },
  { name: 'Athletics', ability: 'strength' },
  { name: 'Deception', ability: 'charisma' },
  { name: 'History', ability: 'intelligence' },
  { name: 'Insight', ability: 'wisdom' },
  { name: 'Intimidation', ability: 'charisma' },
  { name: 'Investigation', ability: 'intelligence' },
  { name: 'Medicine', ability: 'wisdom' },
  { name: 'Nature', ability: 'intelligence' },
  { name: 'Perception', ability: 'wisdom' },
  { name: 'Performance', ability: 'charisma' },
  { name: 'Persuasion', ability: 'charisma' },
  { name: 'Religion', ability: 'intelligence' },
  { name: 'Sleight of Hand', ability: 'dexterity' },
  { name: 'Stealth', ability: 'dexterity' },
  { name: 'Survival', ability: 'wisdom' }
]

// Check if proficient in a skill
const isProficientSkill = (skillName: string): boolean => {
  if (!data.value) return false
  return data.value.proficiencies.skills.some(
    s => s.toLowerCase() === skillName.toLowerCase()
  )
}

// Check if proficient in a save
const isProficientSave = (ability: string): boolean => {
  if (!data.value) return false
  return data.value.proficiencies.saves.some(
    s => s.toLowerCase() === ability.toLowerCase()
  )
}

// Get skill bonus
const getSkillBonus = (skill: { name: string; ability: string }): number => {
  if (!data.value) return 0
  const abilityScore = data.value.abilities[skill.ability as keyof typeof data.value.abilities]
  const mod = getModifier(abilityScore)
  const prof = isProficientSkill(skill.name) ? proficiencyBonus.value : 0
  return mod + prof
}

// Get save bonus
const getSaveBonus = (ability: string, score: number): number => {
  const mod = getModifier(score)
  const prof = isProficientSave(ability) ? proficiencyBonus.value : 0
  return mod + prof
}

// Passive Perception
const passivePerception = computed(() => {
  if (!data.value) return 10
  const wisMod = getModifier(data.value.abilities.wisdom)
  const prof = isProficientSkill('Perception') ? proficiencyBonus.value : 0
  return 10 + wisMod + prof
})

// Attacks
const hasAttacks = computed(() => {
  if (!data.value) return false
  return data.value.equipped.main_hand || data.value.equipped.off_hand
})

const attacks = computed(() => {
  if (!data.value) return []

  const result: { name: string; attackBonus: number; damage: string }[] = []
  const strMod = getModifier(data.value.abilities.strength)
  const dexMod = getModifier(data.value.abilities.dexterity)
  const prof = proficiencyBonus.value

  const getWeaponDamage = (weapon: string, abilityMod: number): string => {
    const w = weapon.toLowerCase()
    if (w.includes('greatsword') || w.includes('maul')) return `2d6${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('greataxe')) return `1d12${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('longsword') || w.includes('warhammer') || w.includes('battleaxe')) return `1d8${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('rapier')) return `1d8${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('shortsword') || w.includes('scimitar')) return `1d6${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('dagger')) return `1d4${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('quarterstaff') || w.includes('spear')) return `1d6${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('longbow')) return `1d8${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('shortbow') || w.includes('light crossbow')) return `1d6${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    if (w.includes('heavy crossbow')) return `1d10${abilityMod >= 0 ? '+' : ''}${abilityMod}`
    return `1d6${abilityMod >= 0 ? '+' : ''}${abilityMod}`
  }

  const isFinesse = (weapon: string): boolean => {
    const w = weapon.toLowerCase()
    return w.includes('rapier') || w.includes('dagger') || w.includes('shortsword') ||
           w.includes('scimitar') || w.includes('whip')
  }

  const isRanged = (weapon: string): boolean => {
    const w = weapon.toLowerCase()
    return w.includes('bow') || w.includes('crossbow') || w.includes('dart') || w.includes('sling')
  }

  // Main hand
  if (data.value.equipped.main_hand) {
    const weapon = data.value.equipped.main_hand
    let abilityMod = strMod
    if (isRanged(weapon)) {
      abilityMod = dexMod
    } else if (isFinesse(weapon) && dexMod > strMod) {
      abilityMod = dexMod
    }
    result.push({
      name: weapon,
      attackBonus: prof + abilityMod,
      damage: getWeaponDamage(weapon, abilityMod)
    })
  }

  // Off hand (if not a shield)
  if (data.value.equipped.off_hand && !data.value.equipped.off_hand.toLowerCase().includes('shield')) {
    const weapon = data.value.equipped.off_hand
    result.push({
      name: weapon,
      attackBonus: prof + strMod,
      damage: getWeaponDamage(weapon, 0) // Off-hand doesn't add ability mod to damage
    })
  }

  return result
})

// Equipment
const hasEquipment = computed(() => {
  if (!data.value) return false
  const e = data.value.equipped
  return e.armor || e.shield || e.main_hand || e.off_hand
})

// Equipment Tab State
const showCurrencyEditor = ref(false)

// Spell edit mode
const isSpellEditMode = ref(false)
const spellEditData = ref<{
  cantrips: { name: string; source: string }[]
  known_spells: { name: string; source: string }[]
} | null>(null)
const showSpellPicker = ref(false)
const spellPickerLevel = ref(0)
const currencyEdit = ref({
  platinum: 0,
  gold: 0,
  electrum: 0,
  silver: 0,
  copper: 0
})

const equippedItems = ref({
  armor: null as string | null,
  shield: null as string | null,
  main_hand: null as string | null,
  off_hand: null as string | null
})

const showAddItem = ref(false)
const newItem = ref({
  name: '',
  source: 'PHB',
  quantity: 1,
  weight: 0,
  notes: ''
})

// Item search state
interface ItemSearchResult {
  name: string
  source: string
  itemType: string
  typeName: string
  rarity: string
  value: number | null
  weight: number | null
}

const itemSearchQuery = ref('')
const itemSearchResults = ref<ItemSearchResult[]>([])
const isSearchingItems = ref(false)
const showItemDropdown = ref(false)
const itemSourceFilter = ref<string>('all')
const availableItemSources = ref<string[]>([])
let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Fetch available item sources on mount
const fetchItemSources = async () => {
  try {
    const sources = await invoke<string[]>('get_item_sources')
    availableItemSources.value = sources.sort()
  } catch (error) {
    console.error('Failed to fetch item sources:', error)
  }
}

// Call on mount
onMounted(async () => {
  await characterStore.getCharacter(characterId.value)
  fetchItemSources()
})

// Rarity order for sorting (common/plain items first)
const rarityOrder: Record<string, number> = {
  'none': 0,
  'common': 1,
  'uncommon': 2,
  'rare': 3,
  'very rare': 4,
  'legendary': 5,
  'artifact': 6
}

// Debounced item search
const searchItems = async (query: string) => {
  if (query.length < 2) {
    itemSearchResults.value = []
    showItemDropdown.value = false
    return
  }

  isSearchingItems.value = true
  try {
    // Use source filter if not 'all'
    const sourceFilter = itemSourceFilter.value === 'all' ? null : [itemSourceFilter.value]
    const results = await invoke<ItemSearchResult[]>('search_items', {
      name: query,
      itemTypes: null,
      rarities: null,
      sources: sourceFilter,
      minValue: null,
      maxValue: null
    })
    // Sort by rarity (common first) then by name
    const sorted = results.sort((a, b) => {
      const rarityA = rarityOrder[a.rarity?.toLowerCase() || 'none'] ?? 0
      const rarityB = rarityOrder[b.rarity?.toLowerCase() || 'none'] ?? 0
      if (rarityA !== rarityB) return rarityA - rarityB
      return a.name.localeCompare(b.name)
    })
    itemSearchResults.value = sorted
    showItemDropdown.value = results.length > 0
  } catch (error) {
    console.error('Failed to search items:', error)
    itemSearchResults.value = []
  } finally {
    isSearchingItems.value = false
  }
}

const onItemSearchInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  itemSearchQuery.value = target.value
  newItem.value.name = target.value

  // Debounce search
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    searchItems(target.value)
  }, 300)
}

const selectSearchItem = (item: ItemSearchResult) => {
  newItem.value.name = item.name
  newItem.value.source = item.source
  newItem.value.weight = item.weight || 0
  itemSearchQuery.value = item.name
  showItemDropdown.value = false
  itemSearchResults.value = []
}

const closeItemDropdown = () => {
  // Delay to allow click on dropdown item
  setTimeout(() => {
    showItemDropdown.value = false
  }, 200)
}

// Initialize equipment state when data loads
watch(() => data.value, (newData) => {
  if (newData) {
    currencyEdit.value = { ...newData.currency }
    equippedItems.value = { ...newData.equipped }
  }
}, { immediate: true })

// Filter inventory by item type
const armorItems = computed(() => {
  if (!data.value) return []
  return data.value.inventory.filter(item => {
    const name = item.name.toLowerCase()
    return name.includes('armor') || name.includes('mail') ||
           name.includes('hide') || name.includes('leather') ||
           name.includes('plate') || name.includes('robe')
  })
})

const shieldItems = computed(() => {
  if (!data.value) return []
  return data.value.inventory.filter(item =>
    item.name.toLowerCase().includes('shield')
  )
})

const weaponItems = computed(() => {
  if (!data.value) return []
  return data.value.inventory.filter(item => {
    const name = item.name.toLowerCase()
    return name.includes('sword') || name.includes('axe') ||
           name.includes('mace') || name.includes('hammer') ||
           name.includes('dagger') || name.includes('bow') ||
           name.includes('crossbow') || name.includes('spear') ||
           name.includes('staff') || name.includes('quarterstaff') ||
           name.includes('rapier') || name.includes('scimitar') ||
           name.includes('flail') || name.includes('whip') ||
           name.includes('trident') || name.includes('halberd') ||
           name.includes('glaive') || name.includes('pike') ||
           name.includes('javelin') || name.includes('club') ||
           name.includes('wand') || name.includes('rod')
  })
})

const offHandItems = computed(() => {
  if (!data.value) return []
  // Off-hand can be shields, light weapons, or wands
  return data.value.inventory.filter(item => {
    const name = item.name.toLowerCase()
    return name.includes('shield') || name.includes('dagger') ||
           name.includes('shortsword') || name.includes('handaxe') ||
           name.includes('light hammer') || name.includes('sickle') ||
           name.includes('wand') || name.includes('rod')
  })
})

// Total weight
const totalWeight = computed(() => {
  if (!data.value) return 0
  return data.value.inventory.reduce((sum, item) => sum + (item.weight * item.quantity), 0)
})

// Currency methods
const saveCurrency = async () => {
  try {
    await invoke('update_character_currency', {
      characterId: characterId.value,
      update: currencyEdit.value
    })
    await characterStore.getCharacter(characterId.value)
    showCurrencyEditor.value = false
  } catch (error) {
    console.error('Failed to update currency:', error)
  }
}

// Equipped items methods
const updateEquipped = async () => {
  try {
    await invoke('update_character_equipped', {
      characterId: characterId.value,
      armor: equippedItems.value.armor,
      shield: equippedItems.value.shield,
      mainHand: equippedItems.value.main_hand,
      offHand: equippedItems.value.off_hand
    })
    await characterStore.getCharacter(characterId.value)
  } catch (error) {
    console.error('Failed to update equipped items:', error)
  }
}

// Inventory methods
const addItem = async () => {
  if (!newItem.value.name) return

  try {
    await invoke('add_item_to_inventory', {
      characterId: characterId.value,
      itemName: newItem.value.name,
      itemSource: newItem.value.source || 'PHB',
      quantity: newItem.value.quantity || 1,
      notes: newItem.value.notes || null
    })
    await characterStore.getCharacter(characterId.value)

    // Reset form
    newItem.value = { name: '', source: 'PHB', quantity: 1, weight: 0, notes: '' }
    itemSearchQuery.value = ''
    itemSearchResults.value = []
    showItemDropdown.value = false
    showAddItem.value = false
  } catch (error) {
    console.error('Failed to add item:', error)
  }
}

const cancelAddItem = () => {
  newItem.value = { name: '', source: 'PHB', quantity: 1, weight: 0, notes: '' }
  itemSearchQuery.value = ''
  itemSearchResults.value = []
  showItemDropdown.value = false
  showAddItem.value = false
}

const incrementItem = async (item: { name: string; source: string | null }) => {
  try {
    await invoke('add_item_to_inventory', {
      characterId: characterId.value,
      itemName: item.name,
      itemSource: item.source || 'PHB',
      quantity: 1,
      notes: null
    })
    await characterStore.getCharacter(characterId.value)
  } catch (error) {
    console.error('Failed to add item:', error)
  }
}

const decrementItem = async (item: { name: string }) => {
  try {
    await invoke('remove_item_from_inventory', {
      characterId: characterId.value,
      itemName: item.name,
      quantity: 1
    })
    await characterStore.getCharacter(characterId.value)
  } catch (error) {
    console.error('Failed to remove item:', error)
  }
}

const removeItemCompletely = async (item: { name: string; quantity: number }) => {
  try {
    await invoke('remove_item_from_inventory', {
      characterId: characterId.value,
      itemName: item.name,
      quantity: item.quantity
    })
    await characterStore.getCharacter(characterId.value)
  } catch (error) {
    console.error('Failed to remove item:', error)
  }
}

// Personality
const hasPersonality = computed(() => {
  if (!data.value) return false
  const p = data.value.personality
  return p.traits || p.ideals || p.bonds || p.flaws
})

// Extended character details
const hasAppearanceData = computed(() => {
  if (!data.value?.appearance) return false
  const a = data.value.appearance
  return a.age || a.height || a.weight || a.eyes || a.hair || a.skin ||
         a.physical_description || a.distinctive_features
})

const hasRoleplayNotes = computed(() => {
  if (!data.value?.roleplay_notes) return false
  const r = data.value.roleplay_notes
  return r.voice_and_mannerisms || r.key_relationships || r.character_goals ||
         r.play_reminders || r.allies_and_organizations || r.additional_treasure_notes
})

// Get player name from store or fall back to stored player_name
const currentPlayerName = computed(() => {
  if (!data.value) return '—'
  if (data.value.player_id) {
    const player = playerStore.getPlayerById(data.value.player_id)
    if (player) return player.name
  }
  // Fall back to stored player_name for backwards compatibility
  return data.value.player_name || '—'
})

const isNpc = computed(() => {
  if (!data.value) return false
  return data.value.npc_role != null ||
         data.value.npc_location != null ||
         data.value.npc_faction != null ||
         data.value.npc_notes != null ||
         (data.value.legendary_actions && data.value.legendary_actions.length > 0)
})

const hasLegendaryActions = computed(() => {
  if (!data.value) return false
  return data.value.legendary_actions && data.value.legendary_actions.length > 0
})

// Spellcasting
const hasSpells = computed(() => {
  if (!data.value) return false
  return data.value.spells.cantrips.length > 0 ||
         data.value.spells.known_spells.length > 0 ||
         data.value.spells.prepared_spells.length > 0 ||
         Object.keys(data.value.spells.spell_slots).length > 0
})

// Sorted spell slot levels
const sortedSpellLevels = computed(() => {
  if (!data.value) return []
  return Object.keys(data.value.spells.spell_slots)
    .map(k => parseInt(k))
    .sort((a, b) => a - b)
})

// Full list casters (prepare from full class list)
const fullListCasters = ['wizard', 'cleric', 'druid', 'paladin']

// Known spells casters (select specific spells at level up)
const knownSpellsCasters = ['bard', 'sorcerer', 'warlock', 'ranger']

// All spellcasting classes (for showing available spells)
const allSpellcasters = [...fullListCasters, ...knownSpellsCasters]

// Check if character has a spellcasting class
const isSpellcaster = computed(() => {
  if (!data.value) return false
  return data.value.classes.some(c =>
    allSpellcasters.includes(c.class_name.toLowerCase())
  )
})

// Check if character has a full list caster class
const isFullListCaster = computed(() => {
  if (!data.value) return false
  return data.value.classes.some(c =>
    fullListCasters.includes(c.class_name.toLowerCase())
  )
})

// Get the primary spellcasting class
const spellcastingClass = computed(() => {
  if (!data.value) return null
  // Find first spellcasting class
  for (const cls of data.value.classes) {
    const name = cls.class_name.toLowerCase()
    if (fullListCasters.includes(name) || knownSpellsCasters.includes(name)) {
      return cls
    }
  }
  return null
})

// Calculate total caster level for multiclass
const totalCasterLevel = computed(() => {
  if (!data.value) return 0

  let casterLevel = 0
  for (const cls of data.value.classes) {
    const className = cls.class_name.toLowerCase()
    const level = cls.level

    // Full casters
    if (['bard', 'cleric', 'druid', 'sorcerer', 'wizard'].includes(className)) {
      casterLevel += level
    }
    // Half casters
    else if (['paladin', 'ranger'].includes(className)) {
      casterLevel += Math.floor(level / 2)
    }
    // Third casters (check subclass)
    else if (className === 'fighter' || className === 'rogue') {
      if (cls.subclass) {
        const subLower = cls.subclass.toLowerCase()
        if (subLower.includes('eldritch knight') || subLower.includes('arcane trickster')) {
          casterLevel += Math.floor(level / 3)
        }
      }
    }
  }

  return casterLevel
})

// Calculate max spell level based on total caster level
const maxSpellLevel = computed(() => {
  const level = totalCasterLevel.value
  if (level === 0) return 0

  if (level >= 17) return 9
  if (level >= 15) return 8
  if (level >= 13) return 7
  if (level >= 11) return 6
  if (level >= 9) return 5
  if (level >= 7) return 4
  if (level >= 5) return 3
  if (level >= 3) return 2
  return 1
})

// Available class spells from catalog
const availableSpells = ref<SpellSummary[]>([])
const loadingSpells = ref(false)

// Calculated spell slots from rules
const calculatedSpellSlots = ref<Record<number, { max: number; current: number }>>({})

// Fetch spell slots from class rules
const fetchSpellSlots = async () => {
  if (!isSpellcaster.value) {
    calculatedSpellSlots.value = {}
    return
  }

  try {
    const slots = await invoke<Record<number, { max: number; current: number }>>('get_character_spell_slots', {
      characterId: characterId.value
    })
    calculatedSpellSlots.value = slots
  } catch (e) {
    console.error('Failed to fetch spell slots:', e)
    calculatedSpellSlots.value = {}
  }
}

// Fetch spells when character loads (only for full list casters)
const fetchClassSpells = async () => {
  if (!isFullListCaster.value || !spellcastingClass.value) {
    availableSpells.value = []
    return
  }

  loadingSpells.value = true
  try {
    // Fetch spells for levels 0 through maxSpellLevel
    const levels = Array.from({ length: maxSpellLevel.value + 1 }, (_, i) => i)

    const spells = await invoke<SpellSummary[]>('search_spells', {
      levels,
      limit: 500
    })

    // Filter by class
    const className = spellcastingClass.value.class_name
    availableSpells.value = spells.filter(spell =>
      spell.classes.some(c => c.toLowerCase() === className.toLowerCase())
    )
  } catch (e) {
    console.error('Failed to fetch class spells:', e)
    availableSpells.value = []
  } finally {
    loadingSpells.value = false
  }
}

// Spells grouped by level
const spellsByLevel = computed(() => {
  const grouped: Record<number, SpellSummary[]> = {}

  for (const spell of availableSpells.value) {
    if (!grouped[spell.level]) {
      grouped[spell.level] = []
    }
    grouped[spell.level].push(spell)
  }

  // Sort spells within each level
  for (const level in grouped) {
    grouped[level].sort((a, b) => a.name.localeCompare(b.name))
  }

  return grouped
})

// Watch for character changes to fetch spells
watch(() => data.value, () => {
  if (data.value) {
    fetchClassSpells()
    fetchSpellSlots()
  }
}, { immediate: true })

// Helper to get primary class name
const primaryClassName = computed(() => {
  if (!data.value || !data.value.classes.length) return ''
  return data.value.classes[0].class_name
})

// Helper to get class string display (e.g., "Fighter 3 / Wizard 2")
const classString = computed(() => {
  if (!data.value || !data.value.classes.length) return ''
  return data.value.classes.map(c => `${c.class_name} ${c.level}`).join(' / ')
})

// Helper to get hit dice string
const hitDiceString = computed(() => {
  if (!data.value || !data.value.classes.length) return ''
  return data.value.classes.map(c => `${c.hit_dice_remaining}${c.hit_dice_type}`).join(', ')
})

// Get spellcasting ability based on class
const spellcastingAbility = computed(() => {
  if (!data.value) return 'intelligence'
  const cls = primaryClassName.value.toLowerCase()
  if (['wizard'].includes(cls)) return 'intelligence'
  if (['cleric', 'druid', 'ranger'].includes(cls)) return 'wisdom'
  if (['bard', 'paladin', 'sorcerer', 'warlock'].includes(cls)) return 'charisma'
  return 'intelligence'
})

const spellSaveDC = computed(() => {
  if (!data.value) return 10
  const ability = data.value.abilities[spellcastingAbility.value as keyof typeof data.value.abilities]
  return 8 + proficiencyBonus.value + getModifier(ability)
})

const spellAttackBonus = computed(() => {
  if (!data.value) return 0
  const ability = data.value.abilities[spellcastingAbility.value as keyof typeof data.value.abilities]
  return proficiencyBonus.value + getModifier(ability)
})

// Hardcoded cantrip progressions as fallback (from PHB)
const cantripProgressionFallback: Record<string, number[]> = {
  wizard: [3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
  cleric: [3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
  druid: [2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
  bard: [2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
  sorcerer: [4, 4, 4, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6],
  warlock: [2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
}

// Hardcoded spells known progressions as fallback (from PHB)
const spellsKnownProgressionFallback: Record<string, number[]> = {
  bard: [4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 15, 16, 18, 19, 19, 20, 22, 22, 22],
  sorcerer: [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 12, 13, 13, 14, 14, 15, 15, 15, 15],
  warlock: [2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15],
  ranger: [0, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11],
}

// Max cantrips known based on class and level (from catalog rules)
const maxCantripsKnown = computed(() => {
  if (!spellcastingClass.value) return 0
  const cls = spellcastingClass.value.class_name.toLowerCase()
  const level = spellcastingClass.value.level

  // Use catalog class data if available (camelCase from JSON)
  if (catalogClassDetails.value?.cantripProgression?.length) {
    const progression = catalogClassDetails.value.cantripProgression
    return progression[Math.min(level, progression.length) - 1] || 0
  }

  // Fallback to hardcoded progression
  const fallback = cantripProgressionFallback[cls]
  if (fallback) {
    return fallback[Math.min(level, 20) - 1] || 0
  }

  // Paladin and Ranger don't get cantrips
  return 0
})

// Max spells known (for known spell casters like Bard, Sorcerer, Warlock, Ranger)
// Full list casters (Wizard, Cleric, Druid, Paladin) don't have a limit on spellbook/known spells
const maxSpellsKnown = computed(() => {
  if (!spellcastingClass.value) return null // null means no limit
  const cls = spellcastingClass.value.class_name.toLowerCase()
  const level = spellcastingClass.value.level

  // Use catalog class data if available (camelCase from JSON)
  if (catalogClassDetails.value?.spellsKnownProgression?.length) {
    const progression = catalogClassDetails.value.spellsKnownProgression
    return progression[Math.min(level, progression.length) - 1] || 0
  }

  // Fallback to hardcoded progression for known casters
  const fallback = spellsKnownProgressionFallback[cls]
  if (fallback) {
    return fallback[Math.min(level, 20) - 1] || 0
  }

  // No progression means it's a full list caster (no limit on spellbook)
  return null
})

// Current counts
const currentCantripsCount = computed(() => {
  return data.value?.spells.cantrips.length || 0
})

const currentSpellsCount = computed(() => {
  return data.value?.spells.known_spells.length || 0
})

// Check if at cantrip limit
const isAtCantripLimit = computed(() => {
  return currentCantripsCount.value >= maxCantripsKnown.value
})

// Check if at spell limit (only for known spell casters)
const isAtSpellLimit = computed(() => {
  if (maxSpellsKnown.value === null) return false // No limit
  return currentSpellsCount.value >= maxSpellsKnown.value
})

// Fetch spell details for rule text
const fetchSpellDetails = async (spellName: string) => {
  if (spellDetails.value[spellName] || loadingSpellDetails.value.has(spellName)) {
    return
  }

  loadingSpellDetails.value.add(spellName)
  try {
    const spell = await invoke<Spell | null>('get_spell_details', {
      name: spellName,
      source: 'PHB'
    })
    if (spell) {
      spellDetails.value[spellName] = spell
    }
  } catch (e) {
    console.error(`Failed to fetch spell details for ${spellName}:`, e)
  } finally {
    loadingSpellDetails.value.delete(spellName)
  }
}

// Toggle spell expansion
const toggleSpellExpansion = async (spellName: string) => {
  if (expandedSpells.value.has(spellName)) {
    expandedSpells.value.delete(spellName)
  } else {
    expandedSpells.value.add(spellName)
    await fetchSpellDetails(spellName)
  }
}

// Fetch item details for stat blocks
const fetchItemDetails = async (itemName: string, itemSource: string | null) => {
  const key = `${itemName}:${itemSource || 'PHB'}`
  if (itemDetails.value[key] || loadingItemDetails.value.has(key)) {
    return
  }

  loadingItemDetails.value.add(key)
  try {
    const item = await invoke<ItemDetails | null>('get_item_details', {
      itemName: itemName,
      itemSource: itemSource || 'PHB'
    })
    if (item) {
      itemDetails.value[key] = item
    }
  } catch (e) {
    console.error(`Failed to fetch item details for ${itemName}:`, e)
  } finally {
    loadingItemDetails.value.delete(key)
  }
}

// Toggle item expansion
const toggleItemExpansion = async (itemName: string, itemSource: string | null) => {
  const key = `${itemName}:${itemSource || 'PHB'}`
  if (expandedItems.value.has(key)) {
    expandedItems.value.delete(key)
  } else {
    expandedItems.value.add(key)
    await fetchItemDetails(itemName, itemSource)
  }
}

// Format item entry to string
const formatItemEntry = (entry: string | object): string => {
  if (typeof entry === 'string') {
    return entry
  }
  if (typeof entry === 'object' && entry !== null) {
    const obj = entry as Record<string, unknown>
    if (obj.type === 'entries' && Array.isArray(obj.entries)) {
      return (obj.entries as Array<string | object>).map(formatItemEntry).join('\n')
    }
    if (obj.type === 'list' && Array.isArray(obj.items)) {
      return (obj.items as string[]).map(item => `  - ${item}`).join('\n')
    }
  }
  return ''
}

// Format spell entry to string
const formatSpellEntry = (entry: string | object): string => {
  if (typeof entry === 'string') {
    return entry
  }
  if (typeof entry === 'object' && entry !== null) {
    const obj = entry as Record<string, unknown>
    if (obj.type === 'entries' && Array.isArray(obj.entries)) {
      return (obj.entries as Array<string | object>).map(formatSpellEntry).join('\n')
    }
    if (obj.type === 'list' && Array.isArray(obj.items)) {
      return (obj.items as string[]).map(item => `  - ${item}`).join('\n')
    }
  }
  return ''
}

// Get all character spells for display
const characterSpellNames = computed(() => {
  if (!data.value) return []
  const names = new Set<string>()

  data.value.spells.cantrips.forEach(s => names.add(s.name))
  data.value.spells.known_spells.forEach(s => names.add(s.name))
  data.value.spells.prepared_spells.forEach(s => names.add(s.name))

  return Array.from(names).sort()
})

// Get spells organized by level for the spell sheet
const spellsForSheet = computed(() => {
  if (!data.value) return {}

  const result: Record<number, string[]> = { 0: [] }

  // Add cantrips
  result[0] = data.value.spells.cantrips.map(s => s.name).sort()

  // For known spells casters, organize by fetched spell data
  // For now, put all known spells in a "Known" section
  const knownSpellNames = data.value.spells.known_spells.map(s => s.name).sort()

  // Try to organize by level using availableSpells data
  for (const spellName of knownSpellNames) {
    const spellInfo = availableSpells.value.find(s => s.name === spellName)
    const level = spellInfo?.level ?? 1
    if (!result[level]) result[level] = []
    result[level].push(spellName)
  }

  return result
})

// Actions
const goBack = () => {
  router.push('/characters')
}

// Level up dialog
const showLevelUpDialog = ref(false)

const levelUp = () => {
  showLevelUpDialog.value = true
}

const handleLevelUpCompleted = async () => {
  // Reload character data after level up
  await characterStore.getCharacter(characterId.value)
}

// Inventory manager
const showInventoryManager = ref(false)

const handleInventoryUpdated = async () => {
  // Reload character data after inventory changes
  await characterStore.getCharacter(characterId.value)
}

// PDF printing
const showPdfPreview = ref(false)
const showPrintDialog = ref(false)
const isPrintingPdf = ref(false)
const pdfPreviewRef = ref<InstanceType<typeof PdfPreviewModal> | null>(null)

const pdfPreviewTitle = computed(() => {
  if (!data.value) return 'Character Sheet'
  return `${data.value.character_name} - Character Sheet`
})

const pdfFileName = computed(() => {
  if (!data.value) return 'character-sheet.pdf'
  const charName = data.value.character_name.replace(/\s+/g, '_')
  const classStr = data.value.classes
    .map(c => `${c.class_name}${c.level}`)
    .join('_')
  return `${charName}_${classStr}.pdf`
})

const printCharacter = () => {
  if (!data.value) return
  showPrintDialog.value = true
}

// Edit mode functions
const startEditing = () => {
  if (!data.value) return
  editData.value = {
    character_name: data.value.character_name,
    alignment: data.value.alignment || '',
    max_hp: data.value.max_hp,
    current_hp: data.value.current_hp,
    abilities: { ...data.value.abilities },
    personality: {
      traits: data.value.personality.traits || '',
      ideals: data.value.personality.ideals || '',
      bonds: data.value.personality.bonds || '',
      flaws: data.value.personality.flaws || ''
    },
    legendary_actions: data.value.legendary_actions
      ? data.value.legendary_actions.map(a => ({ ...a }))
      : [],
    legendary_action_count: data.value.legendary_action_count || 3,
    // Extended character details
    player_id: data.value.player_id,
    appearance: {
      age: data.value.appearance?.age || '',
      height: data.value.appearance?.height || '',
      weight: data.value.appearance?.weight || '',
      eyes: data.value.appearance?.eyes || '',
      hair: data.value.appearance?.hair || '',
      skin: data.value.appearance?.skin || '',
      physical_description: data.value.appearance?.physical_description || '',
      distinctive_features: data.value.appearance?.distinctive_features || ''
    },
    backstory: data.value.backstory || '',
    background_feature: data.value.background_feature || '',
    roleplay_notes: {
      voice_and_mannerisms: data.value.roleplay_notes?.voice_and_mannerisms || '',
      key_relationships: data.value.roleplay_notes?.key_relationships || '',
      character_goals: data.value.roleplay_notes?.character_goals || '',
      play_reminders: data.value.roleplay_notes?.play_reminders || '',
      allies_and_organizations: data.value.roleplay_notes?.allies_and_organizations || '',
      additional_treasure_notes: data.value.roleplay_notes?.additional_treasure_notes || ''
    }
  }
  isEditing.value = true
}

const cancelEditing = () => {
  isEditing.value = false
  editData.value = null
}

// Legendary action edit helpers
const addLegendaryActionEdit = () => {
  if (!editData.value) return
  editData.value.legendary_actions.push({
    name: '',
    cost: 1,
    description: ''
  })
}

const removeLegendaryActionEdit = (index: number) => {
  if (!editData.value) return
  editData.value.legendary_actions.splice(index, 1)
}

const saveEdits = async () => {
  if (!data.value || !editData.value) return

  try {
    // Create updated character data
    const updatedData = {
      ...data.value,
      character_name: editData.value.character_name,
      alignment: editData.value.alignment || null,
      max_hp: editData.value.max_hp,
      current_hp: editData.value.current_hp,
      abilities: editData.value.abilities,
      personality: {
        traits: editData.value.personality.traits || null,
        ideals: editData.value.personality.ideals || null,
        bonds: editData.value.personality.bonds || null,
        flaws: editData.value.personality.flaws || null
      },
      legendary_actions: editData.value.legendary_actions.filter(a => a.name.trim() !== ''),
      legendary_action_count: editData.value.legendary_actions.length > 0
        ? editData.value.legendary_action_count
        : null,
      // Extended character details
      player_id: editData.value.player_id,
      // Derive player_name from selected player for PDF export
      player_name: editData.value.player_id
        ? playerStore.getPlayerById(editData.value.player_id)?.name || null
        : null,
      appearance: {
        age: editData.value.appearance.age || null,
        height: editData.value.appearance.height || null,
        weight: editData.value.appearance.weight || null,
        eyes: editData.value.appearance.eyes || null,
        hair: editData.value.appearance.hair || null,
        skin: editData.value.appearance.skin || null,
        physical_description: editData.value.appearance.physical_description || null,
        distinctive_features: editData.value.appearance.distinctive_features || null
      },
      backstory: editData.value.backstory || null,
      background_feature: editData.value.background_feature || null,
      roleplay_notes: {
        voice_and_mannerisms: editData.value.roleplay_notes.voice_and_mannerisms || null,
        key_relationships: editData.value.roleplay_notes.key_relationships || null,
        character_goals: editData.value.roleplay_notes.character_goals || null,
        play_reminders: editData.value.roleplay_notes.play_reminders || null,
        allies_and_organizations: editData.value.roleplay_notes.allies_and_organizations || null,
        additional_treasure_notes: editData.value.roleplay_notes.additional_treasure_notes || null
      }
    }

    await invoke('update_character', {
      characterId: characterId.value,
      characterData: updatedData,
      snapshotReason: 'Manual edit'
    })

    // Reload character data
    await characterStore.getCharacter(characterId.value)
    isEditing.value = false
    editData.value = null
  } catch (e) {
    console.error('Failed to save character:', e)
    alert('Failed to save character: ' + e)
  }
}

// Export character sheet as markdown
const exportCharacter = async () => {
  try {
    const markdown = await invoke<string>('render_character_sheet', {
      characterId: characterId.value
    })

    // Use Tauri's save dialog
    const { save } = await import('@tauri-apps/plugin-dialog')

    // Build filename: CharacterName_Class1Level_Class2Level.md
    const charName = data.value?.character_name || 'character'
    const classStr = data.value?.classes
      .map(c => `${c.class_name}${c.level}`)
      .join('_') || 'Unknown'
    const filename = `${charName}_${classStr}.md`

    const filePath = await save({
      defaultPath: filename,
      filters: [{ name: 'Markdown', extensions: ['md'] }]
    })

    if (filePath) {
      // Write file via backend command
      await invoke('write_text_file', {
        path: filePath,
        contents: markdown
      })
    }
  } catch (e) {
    console.error('Failed to export character:', e)
    alert('Failed to export character: ' + e)
  }
}

const deleteCharacter = async () => {
  if (!confirm('Are you sure you want to delete this character?')) return

  try {
    await characterStore.deleteCharacter(characterId.value)
    router.push('/characters')
  } catch (e) {
    console.error('Failed to delete character:', e)
  }
}

// Spell edit mode functions
const startSpellEdit = () => {
  if (!data.value) return
  spellEditData.value = {
    cantrips: [...data.value.spells.cantrips],
    known_spells: [...data.value.spells.known_spells]
  }
  isSpellEditMode.value = true
}

const cancelSpellEdit = () => {
  isSpellEditMode.value = false
  spellEditData.value = null
  showSpellPicker.value = false
}

const removeSpell = async (spellName: string, isCantrip: boolean) => {
  if (!data.value) return

  try {
    // Update local data
    const updatedSpells = {
      ...data.value.spells,
      cantrips: isCantrip
        ? data.value.spells.cantrips.filter(s => s.name !== spellName)
        : data.value.spells.cantrips,
      known_spells: !isCantrip
        ? data.value.spells.known_spells.filter(s => s.name !== spellName)
        : data.value.spells.known_spells
    }

    const updatedData = {
      ...data.value,
      spells: updatedSpells
    }

    await invoke('update_character', {
      characterId: characterId.value,
      characterData: updatedData,
      snapshotReason: 'Removed spell'
    })

    // Reload character data
    await characterStore.getCharacter(characterId.value)
  } catch (e) {
    console.error('Failed to remove spell:', e)
    alert('Failed to remove spell: ' + e)
  }
}

const openSpellPicker = (level: number) => {
  spellPickerLevel.value = level
  showSpellPicker.value = true
}

const addSpell = async (spell: SpellSummary) => {
  if (!data.value) return

  try {
    const spellRef = { name: spell.name, source: spell.source }
    const isCantrip = spell.level === 0

    // Check if spell already exists
    const existingSpells = isCantrip ? data.value.spells.cantrips : data.value.spells.known_spells
    if (existingSpells.some(s => s.name === spell.name)) {
      alert('Spell already known')
      return
    }

    // Update local data
    const updatedSpells = {
      ...data.value.spells,
      cantrips: isCantrip
        ? [...data.value.spells.cantrips, spellRef]
        : data.value.spells.cantrips,
      known_spells: !isCantrip
        ? [...data.value.spells.known_spells, spellRef]
        : data.value.spells.known_spells
    }

    const updatedData = {
      ...data.value,
      spells: updatedSpells
    }

    await invoke('update_character', {
      characterId: characterId.value,
      characterData: updatedData,
      snapshotReason: 'Added spell'
    })

    // Reload character data
    await characterStore.getCharacter(characterId.value)
    showSpellPicker.value = false
  } catch (e) {
    console.error('Failed to add spell:', e)
    alert('Failed to add spell: ' + e)
  }
}

// Check if a spell is already known by the character
const isSpellKnown = (spellName: string): boolean => {
  if (!data.value) return false
  const allKnown = [
    ...data.value.spells.cantrips.map(s => s.name),
    ...data.value.spells.known_spells.map(s => s.name)
  ]
  return allKnown.includes(spellName)
}

// Get sorted spell names for a level (known spells first, then alphabetical)
const getSortedSpellsForLevel = (level: number): string[] => {
  if (!isFullListCaster.value) {
    // For known spell casters, just return their known spells
    if (level === 0) {
      return data.value?.spells.cantrips.map(s => s.name) || []
    }
    return spellsForSheet.value[level] || []
  }

  // For full list casters, sort with known first
  const allSpells = spellsByLevel.value[level]?.map(s => s.name) || []
  return allSpells.sort((a, b) => {
    const aKnown = isSpellKnown(a)
    const bKnown = isSpellKnown(b)
    if (aKnown && !bKnown) return -1
    if (!aKnown && bKnown) return 1
    return a.localeCompare(b)
  })
}

// Toggle a spell in/out of the spellbook (for full list casters)
const toggleSpellInBook = async (spellName: string, isCantrip: boolean) => {
  if (!data.value) return

  const isKnown = isSpellKnown(spellName)

  // Check limits before adding
  if (!isKnown) {
    if (isCantrip && isAtCantripLimit.value) {
      // At cantrip limit, don't add
      return
    }
    if (!isCantrip && isAtSpellLimit.value) {
      // At spell limit, don't add
      return
    }
  }

  // Update local data immediately (optimistic update)
  if (isKnown) {
    if (isCantrip) {
      data.value.spells.cantrips = data.value.spells.cantrips.filter(s => s.name !== spellName)
    } else {
      data.value.spells.known_spells = data.value.spells.known_spells.filter(s => s.name !== spellName)
    }
  } else {
    const spellData = availableSpells.value.find(s => s.name === spellName)
    const source = spellData?.source || 'PHB'
    const spellRef = { name: spellName, source }
    if (isCantrip) {
      data.value.spells.cantrips = [...data.value.spells.cantrips, spellRef]
    } else {
      data.value.spells.known_spells = [...data.value.spells.known_spells, spellRef]
    }
  }

  // Save to backend in background (don't await to prevent scroll jump)
  invoke('update_character', {
    characterId: characterId.value,
    characterData: data.value,
    snapshotReason: isKnown ? 'Removed spell from spellbook' : 'Added spell to spellbook'
  }).catch(e => {
    console.error('Failed to save spell change:', e)
  })
}

// Filter available spells by level for picker
const spellsForPicker = computed(() => {
  if (!spellcastingClass.value) return []

  const className = spellcastingClass.value.class_name.toLowerCase()
  return availableSpells.value
    .filter(spell => {
      // Filter by level
      if (spell.level !== spellPickerLevel.value) return false
      // Filter by class (if available in spell data)
      if (spell.classes && spell.classes.length > 0) {
        return spell.classes.some(c => c.toLowerCase() === className)
      }
      return true
    })
    .sort((a, b) => a.name.localeCompare(b.name))
})
</script>

<style scoped>
.character-sheet-view {
  @apply space-y-6;
}

.loading,
.error-message {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-secondary);
}

.error-message {
  color: var(--color-error);
}

/* Header */
.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding-bottom: var(--spacing-lg);
  border-bottom: 2px solid var(--color-border);
}

.header-main {
  @apply space-y-1;
}

.btn-back {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 0.875rem;
  margin-bottom: var(--spacing-sm);
}

.btn-back:hover {
  background: var(--color-surface);
}

.character-name {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-text);
}

.character-subtitle {
  font-size: 1.25rem;
  color: var(--color-primary-500);
  font-weight: 500;
}

.character-background {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

/* Tab Navigation */
.tab-navigation {
  display: flex;
  gap: var(--spacing-sm);
  margin: var(--spacing-md) 0;
  border-bottom: 2px solid var(--color-border);
  padding-bottom: var(--spacing-sm);
}

.tab-button {
  padding: var(--spacing-sm) var(--spacing-lg);
  background: transparent;
  border: none;
  border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  color: var(--color-text-secondary);
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-button:hover {
  color: var(--color-text);
  background: var(--color-surface-variant);
}

.tab-button.active {
  color: var(--color-primary-500);
  background: var(--color-surface-variant);
  border-bottom: 2px solid var(--color-primary-500);
  margin-bottom: -2px;
}

.spell-note {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-style: italic;
  margin-top: var(--spacing-sm);
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.btn-secondary,
.btn-danger {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  border: none;
  font-weight: 500;
  cursor: pointer;
}

.btn-secondary {
  background: var(--color-surface-variant);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-primary-100);
}

.btn-danger {
  background: var(--color-error);
  color: white;
}

.btn-danger:hover {
  opacity: 0.9;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  border: none;
  font-weight: 500;
  cursor: pointer;
}

.btn-primary:hover {
  opacity: 0.9;
}

/* Edit mode inputs */
.edit-input {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  color: var(--color-text);
  font-family: inherit;
}

.edit-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.edit-name {
  font-size: 1.5rem;
  font-weight: 700;
  width: 100%;
  margin-bottom: var(--spacing-xs);
}

.edit-alignment {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.edit-alignment-input {
  width: 150px;
  background: var(--color-surface);
  border: 2px solid var(--color-primary-100);
}

.edit-hp {
  width: 48px;
  height: 28px;
  text-align: center;
  padding: 2px 4px;
  -moz-appearance: textfield;
}

.edit-hp::-webkit-outer-spin-button,
.edit-hp::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.hp-edit-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.edit-ability {
  width: 44px;
  height: 36px;
  text-align: center;
  font-size: 1.25rem;
  font-weight: 700;
  padding: 4px;
  margin: 0 auto;
  display: block;
  -moz-appearance: textfield;
}

.edit-ability::-webkit-outer-spin-button,
.edit-ability::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.edit-textarea {
  width: 100%;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-sm);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.875rem;
  resize: vertical;
  margin-top: var(--spacing-xs);
}

.edit-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
}

/* Main Content */
.sheet-content {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-lg);
}

@media (max-width: 1024px) {
  .sheet-content {
    grid-template-columns: 1fr;
  }
}

.sheet-column {
  @apply space-y-4;
}

.sheet-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-md);
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
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
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.ability-score {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text);
}

.ability-modifier {
  font-size: 0.875rem;
  color: var(--color-primary-500);
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

.combat-stat .stat-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.combat-stat .stat-value {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
}

.combat-stat .stat-note {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
}

.ac-shield {
  color: var(--color-primary-500);
}

.hp-stat {
  grid-column: span 2;
}

/* Saves & Skills */
.saves-list,
.skills-list {
  @apply space-y-1;
}

.save-item,
.skill-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) 0;
  font-size: 0.875rem;
}

.save-proficient,
.skill-proficient {
  width: 1rem;
  color: var(--color-text-secondary);
  opacity: 0.3;
}

.save-proficient.active,
.skill-proficient.active {
  color: var(--color-primary-500);
  opacity: 1;
}

.save-name,
.skill-name {
  flex: 1;
  color: var(--color-text);
  text-transform: capitalize;
}

.skill-ability {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.save-bonus,
.skill-bonus {
  font-weight: 500;
  color: var(--color-primary-500);
  min-width: 2rem;
  text-align: right;
}

/* Attacks */
.attacks-list {
  @apply space-y-2;
}

.attack-item {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
}

.attack-name {
  font-weight: 500;
  color: var(--color-text);
}

.attack-bonus {
  font-weight: 600;
  color: var(--color-primary-500);
}

.attack-damage {
  color: var(--color-text-secondary);
}

/* Proficiencies */
.proficiency-group {
  font-size: 0.875rem;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.proficiency-group strong {
  color: var(--color-text-secondary);
}

/* Features & Equipment */
.feature-list,
.equipment-list {
  @apply space-y-1;
  padding-left: var(--spacing-md);
  font-size: 0.875rem;
  color: var(--color-text);
}

.feature-item {
  margin-bottom: var(--spacing-sm);
}

.feature-header {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.feature-header:hover {
  color: var(--color-primary);
}

.feature-source {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.expand-icon {
  font-size: 0.625rem;
  color: var(--color-text-secondary);
  margin-left: auto;
}

.feature-description {
  margin-top: var(--spacing-xs);
  padding: var(--spacing-sm);
  background: var(--color-surface-elevated);
  border-radius: var(--radius-sm);
  font-size: 0.8125rem;
  line-height: 1.5;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
}

.loading-indicator {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-style: italic;
  padding: var(--spacing-xs) 0;
}

.equipped-items {
  margin-bottom: var(--spacing-md);
}

.equipped-item {
  font-size: 0.875rem;
  padding: var(--spacing-xs) 0;
}

.equipped-slot {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.inventory-section {
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-sm);
}

.inventory-label {
  display: block;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xs);
}

/* Spells */
.spell-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-md);
}

.spell-stat {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.spell-stat .stat-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.spell-stat .stat-value {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
}

.spell-slots {
  margin-bottom: var(--spacing-md);
  font-size: 0.875rem;
}

.spell-slots strong {
  color: var(--color-text-secondary);
  display: block;
  margin-bottom: var(--spacing-xs);
}

.slots-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
}

.slot-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  min-width: 3rem;
}

.slot-level {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
}

.slot-count {
  font-weight: 600;
  color: var(--color-primary-500);
}

.spell-group {
  font-size: 0.875rem;
  margin-bottom: var(--spacing-sm);
}

.spell-group strong {
  color: var(--color-text-secondary);
}

/* Available spells for full list casters */
.available-spells {
  margin-top: var(--spacing-md);
}

.loading-spells {
  color: var(--color-text-secondary);
  font-style: italic;
  font-size: 0.875rem;
}

.spell-level-group {
  margin-bottom: var(--spacing-md);
}

.level-header {
  display: block;
  font-size: 0.875rem;
  color: var(--color-primary-500);
  margin-bottom: var(--spacing-xs);
}

.spell-names {
  font-size: 0.8rem;
  color: var(--color-text);
  line-height: 1.4;
}

.spell-tag {
  font-size: 0.6rem;
  vertical-align: super;
  color: var(--color-text-secondary);
  margin-left: 1px;
}

.no-spells {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

/* Currency */
.currency-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  font-size: 0.875rem;
}

.currency-item strong {
  color: var(--color-text-secondary);
}

/* Personality */
.personality-item {
  font-size: 0.875rem;
  margin-bottom: var(--spacing-sm);
  line-height: 1.4;
}

.personality-item strong {
  color: var(--color-text-secondary);
  display: block;
  margin-bottom: 2px;
}

/* Legendary Actions Section */
.legendary-intro {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-md);
  line-height: 1.5;
}

.legendary-action-view {
  font-size: 0.875rem;
  margin-bottom: var(--spacing-sm);
  line-height: 1.4;
}

.legendary-count-edit {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.legendary-count-edit label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.legendary-action-edit {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-sm);
}

.action-edit-header {
  display: flex;
  gap: var(--spacing-sm);
  align-items: center;
  margin-bottom: var(--spacing-xs);
}

.action-edit-header .edit-input {
  flex: 1;
}

.action-cost-edit {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.action-cost-edit label {
  color: var(--color-text-secondary);
}

.form-select-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
  min-width: 60px;
}

.form-select-sm:focus {
  outline: none;
  border-color: var(--color-primary);
}

.btn-add-legendary {
  padding: var(--spacing-xs) var(--spacing-md);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.btn-add-legendary:hover {
  background: var(--color-primary-600);
}

.btn-remove-legendary {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.btn-remove-legendary:hover {
  background: var(--color-error);
  color: white;
  border-color: var(--color-error);
}

.action-desc-edit {
  min-height: 50px;
}

.no-actions-hint {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Spells Sheet Tab */
.spells-sheet {
  @apply space-y-6;
}

.spells-header {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
}

.spellcasting-info {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.spell-stat-box {
  text-align: center;
  padding: var(--spacing-sm);
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border);
}

.spell-stat-box .stat-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  color: var(--color-text-secondary);
  display: block;
  margin-bottom: var(--spacing-xs);
}

.spell-stat-box .stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text);
}

.spell-slots-row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  justify-content: center;
}

.spell-slot-box {
  text-align: center;
  padding: var(--spacing-sm);
  min-width: 4rem;
}

.slot-level-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  display: block;
  margin-bottom: var(--spacing-xs);
}

.slot-circles {
  display: flex;
  gap: 4px;
  justify-content: center;
}

.slot-circle {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--color-primary-500);
  background: var(--color-primary-500);
}

.slot-circle.used {
  background: transparent;
}

.spell-levels-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--spacing-lg);
}

.spell-level-section {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.level-header-box {
  background: var(--color-primary-700);
  color: white;
  padding: var(--spacing-sm) var(--spacing-md);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.level-title {
  font-weight: 600;
  font-size: 0.9rem;
}

.slots-remaining {
  font-size: 0.8rem;
  opacity: 0.8;
}

.level-slots-info {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.slots-total {
  font-size: 0.75rem;
  opacity: 0.9;
}

.slots-circles-header {
  display: flex;
  gap: 3px;
}

.slot-circle-sm {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.8);
}

.slot-circle-sm.used {
  background: transparent;
}

.spell-list {
  padding: var(--spacing-sm);
  max-height: 400px;
  overflow-y: auto;
}

.spell-item {
  border-bottom: 1px solid var(--color-border);
}

.spell-item:last-child {
  border-bottom: none;
}

.spell-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm);
  cursor: pointer;
  transition: background 0.2s;
}

.spell-row:hover {
  background: var(--color-surface);
}

.spell-item .spell-name {
  font-size: 0.875rem;
  color: var(--color-text);
}

.expand-icon {
  font-size: 1rem;
  color: var(--color-text-secondary);
  font-weight: bold;
}

/* Spell edit mode styles */
.spell-checkbox {
  width: 20px;
  height: 20px;
  min-width: 20px;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: var(--spacing-sm);
  background: var(--color-background);
  font-size: 0.875rem;
  font-weight: bold;
  transition: all 0.15s ease;
}

.spell-checkbox.checked {
  background: var(--color-primary-500);
  border-color: var(--color-primary-500);
  color: white;
}

.spell-in-book {
  background: var(--color-primary-50);
}

.spell-in-book .spell-name {
  color: var(--color-primary-700);
  font-weight: 500;
}

.edit-hint {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  font-style: italic;
  opacity: 0.8;
}

.spell-count {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-left: var(--spacing-xs);
}

.spell-count.at-limit,
.stat-value.at-limit,
.slots-total.at-limit {
  color: var(--color-warning-600);
  font-weight: 600;
}

.btn-add-spell:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.spell-details {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
}

.loading-details {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.spell-meta {
  display: flex;
  gap: var(--spacing-md);
  align-items: center;
  margin-bottom: var(--spacing-sm);
  font-size: 0.8rem;
  color: var(--color-primary-500);
  font-style: italic;
}

.concentration-tag {
  background: var(--color-warning-100);
  color: var(--color-warning-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.7rem;
  font-style: normal;
}

.spell-properties {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
  font-size: 0.8rem;
}

.spell-properties div {
  color: var(--color-text-secondary);
}

.spell-properties strong {
  color: var(--color-text);
}

.spell-description {
  font-size: 0.8rem;
  line-height: 1.5;
  color: var(--color-text);
}

.spell-description p {
  margin-bottom: var(--spacing-sm);
}

.spell-description p:last-child {
  margin-bottom: 0;
}

.no-spells-message {
  padding: var(--spacing-sm);
  text-align: center;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

/* Spell edit mode styles */
.spells-header-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: var(--spacing-sm);
}

.btn-add-spell {
  background: var(--color-primary-500);
  color: white;
  border: none;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  cursor: pointer;
  margin-left: auto;
}

.btn-add-spell:hover {
  background: var(--color-primary-600);
}

.btn-remove-spell {
  background: var(--color-error);
  color: white;
  border: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  margin-right: var(--spacing-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.btn-remove-spell:hover {
  background: var(--color-error-dark, #c82333);
}

/* Spell picker modal */
.spell-picker-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.spell-picker-modal {
  background: var(--color-bg);
  border-radius: var(--radius-lg);
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.spell-picker-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.spell-picker-header h3 {
  margin: 0;
  font-size: 1.1rem;
}

.btn-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: var(--color-text-secondary);
  padding: 0;
  line-height: 1;
}

.btn-close:hover {
  color: var(--color-text);
}

.spell-picker-content {
  padding: var(--spacing-md);
  overflow-y: auto;
  flex: 1;
}

.no-spells-available {
  text-align: center;
  color: var(--color-text-secondary);
  padding: var(--spacing-lg);
}

.spell-picker-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.spell-picker-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-align: left;
  transition: background 0.2s, border-color 0.2s;
}

.spell-picker-item:hover {
  background: var(--color-surface-hover, var(--color-bg));
  border-color: var(--color-primary-500);
}

.picker-spell-name {
  font-weight: 500;
  color: var(--color-text);
}

.picker-spell-school {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.picker-spell-known-badge {
  font-size: 0.75rem;
  background: var(--color-success, #28a745);
  color: white;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-weight: 500;
}

.spell-picker-item.spell-known {
  opacity: 0.6;
  cursor: default;
  background: var(--color-surface);
}

.spell-picker-item.spell-known:hover {
  border-color: var(--color-border);
  background: var(--color-surface);
}

/* Item styles */
.item-list {
  padding: 0;
}

.item-entry {
  border-bottom: 1px solid var(--color-border);
}

.item-entry:last-child {
  border-bottom: none;
}

.item-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) 0;
  cursor: pointer;
  transition: background 0.2s;
}

.item-row:hover {
  background: var(--color-surface);
}

.item-name {
  font-size: 0.875rem;
  color: var(--color-text);
}

.item-details {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
}

.item-notes {
  padding: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
  background: var(--color-warning-100);
  border-radius: var(--radius-sm);
  font-size: 0.8rem;
  color: var(--color-text);
}

.item-meta {
  display: flex;
  gap: var(--spacing-md);
  align-items: center;
  margin-bottom: var(--spacing-sm);
  font-size: 0.8rem;
  color: var(--color-primary-500);
  font-style: italic;
}

.item-rarity {
  text-transform: capitalize;
}

.item-properties {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
  font-size: 0.8rem;
}

.item-properties div {
  color: var(--color-text-secondary);
}

.item-properties strong {
  color: var(--color-text);
}

.item-description {
  font-size: 0.8rem;
  line-height: 1.5;
  color: var(--color-text);
}

.item-description p {
  margin-bottom: var(--spacing-sm);
}

.item-description p:last-child {
  margin-bottom: 0;
}

/* Equipment Tab */
.equipment-sheet {
  padding: var(--spacing-lg);
}

.equipment-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 800px;
}

.equipment-section {
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  border: 1px solid var(--color-border);
}

.equipment-section .section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.equipment-section .section-title {
  margin: 0;
  font-size: 1.125rem;
}

/* Currency Section */
.currency-display {
  display: flex;
  gap: var(--spacing-md);
  flex-wrap: wrap;
}

.currency-display .currency-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  min-width: 60px;
}

.currency-display .currency-item.large {
  min-width: 80px;
}

.currency-icon {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.currency-icon.gold {
  color: #ffc107;
}

.currency-icon.silver {
  color: #9e9e9e;
}

.currency-icon.copper {
  color: #cd7f32;
}

.currency-value {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text);
}

.currency-editor {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  align-items: flex-end;
}

.currency-input-row {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.currency-input-row label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.currency-input-row input {
  width: 80px;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 1rem;
}

.btn-edit-small {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.btn-edit-small:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

/* Equipped Slots */
.equipped-slots {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.equip-slot {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.slot-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.equip-slot select {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

/* Inventory Section */
.btn-add {
  padding: var(--spacing-xs) var(--spacing-md);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  cursor: pointer;
}

.btn-add:hover {
  background: var(--color-primary-600);
}

.add-item-form {
  background: var(--color-surface-variant);
  padding: var(--spacing-md);
  border-radius: var(--radius-md);
  margin-bottom: var(--spacing-md);
}

.add-item-form .form-row {
  display: flex;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.add-item-form input {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.input-name {
  flex: 2;
}

/* Search Input with Dropdown */
.search-input-container {
  position: relative;
  flex: 2;
}

.search-input-container .input-name {
  width: 100%;
}

.search-spinner {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.search-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-top: none;
  border-radius: 0 0 var(--radius-sm) var(--radius-sm);
  max-height: 300px;
  overflow-y: auto;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.search-result-item {
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
  border-bottom: 1px solid var(--color-border);
}

.search-result-item:last-child {
  border-bottom: none;
}

.search-result-item:hover {
  background: var(--color-surface-variant);
}

.result-name {
  font-weight: 500;
  color: var(--color-text);
}

.result-meta {
  display: flex;
  gap: var(--spacing-sm);
  font-size: 0.75rem;
}

.result-type {
  color: var(--color-text-secondary);
}

.result-rarity {
  color: var(--color-primary-500);
  font-weight: 500;
}

.result-source {
  color: var(--color-text-secondary);
  opacity: 0.7;
}

.no-results {
  padding: var(--spacing-md);
  text-align: center;
  color: var(--color-text-secondary);
  font-style: italic;
}

.input-qty {
  width: 70px;
}

.input-source-filter {
  width: 120px;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.input-notes {
  flex: 1;
}

.form-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
}

.btn-small {
  padding: var(--spacing-xs) var(--spacing-md);
  font-size: 0.875rem;
}

.inventory-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.inventory-item {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.inventory-item .item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
}

.inventory-item .item-header:hover {
  background: var(--color-surface-elevated);
}

.item-main {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.inventory-item .item-name {
  font-weight: 500;
  color: var(--color-text);
}

.item-qty {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.item-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.btn-qty {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  cursor: pointer;
  font-size: 1rem;
}

.btn-qty:hover {
  background: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
}

.btn-remove {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-sm);
  color: var(--color-danger);
  cursor: pointer;
  font-size: 0.875rem;
}

.btn-remove:hover {
  background: var(--color-danger);
  color: white;
}

.inventory-item .item-details {
  padding: var(--spacing-sm) var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.item-meta-row {
  display: flex;
  gap: var(--spacing-md);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.catalog-details {
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px dashed var(--color-border);
}

.item-type-rarity {
  display: flex;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.item-type {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.item-rarity {
  font-size: 0.75rem;
  color: var(--color-primary-500);
  font-weight: 500;
}

.item-stats {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  font-size: 0.8rem;
  margin-bottom: var(--spacing-xs);
}

.item-properties {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.inventory-footer {
  margin-top: var(--spacing-md);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-border);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.total-weight {
  display: flex;
  gap: var(--spacing-xs);
}

/* Details Tab */
.details-sheet {
  padding: var(--spacing-lg);
}

.details-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 800px;
}

.details-section {
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  border: 1px solid var(--color-border);
}

.details-section .section-title {
  margin: 0 0 var(--spacing-md) 0;
  font-size: 1.125rem;
  color: var(--color-text);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: var(--spacing-sm);
}

/* Player Info */
.player-info-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.player-info-row .field-label {
  font-weight: 600;
  color: var(--color-text-secondary);
  min-width: 100px;
}

.player-info-row .field-value {
  color: var(--color-text);
}

.player-info-row .edit-input,
.player-info-row .edit-select {
  flex: 1;
  max-width: 300px;
}

.player-info-row .edit-select {
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
  cursor: pointer;
}

.player-info-row .edit-select:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-alpha);
}

/* Appearance Section */
.appearance-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.appearance-field {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.appearance-field label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.appearance-field .edit-input {
  width: 100%;
}

.appearance-textarea-section {
  margin-top: var(--spacing-md);
}

.appearance-textarea-section label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.appearance-grid-view {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-md);
}

.appearance-item {
  font-size: 0.875rem;
}

.appearance-item strong {
  color: var(--color-text-secondary);
}

.appearance-text {
  margin-top: var(--spacing-md);
}

.appearance-text strong {
  display: block;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.appearance-text p {
  margin: 0;
  line-height: 1.5;
}

/* Background Section */
.background-field {
  margin-bottom: var(--spacing-md);
}

.background-field:last-child {
  margin-bottom: 0;
}

.background-field label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.background-text {
  margin-bottom: var(--spacing-md);
}

.background-text:last-child {
  margin-bottom: 0;
}

.background-text strong {
  display: block;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.background-text p {
  margin: 0;
  line-height: 1.5;
}

.backstory-text {
  white-space: pre-wrap;
}

/* Roleplay Notes Section */
.roleplay-field {
  margin-bottom: var(--spacing-md);
}

.roleplay-field:last-child {
  margin-bottom: 0;
}

.roleplay-field label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.roleplay-item {
  margin-bottom: var(--spacing-md);
}

.roleplay-item:last-child {
  margin-bottom: 0;
}

.roleplay-item strong {
  display: block;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  text-transform: uppercase;
  margin-bottom: var(--spacing-xs);
}

.roleplay-item p {
  margin: 0;
  line-height: 1.5;
}

/* Responsive adjustments for appearance grid */
@media (max-width: 600px) {
  .appearance-grid,
  .appearance-grid-view {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>

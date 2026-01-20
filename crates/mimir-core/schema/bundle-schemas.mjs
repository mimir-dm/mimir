/**
 * Bundle 5etools JSON schemas by dereferencing all $ref pointers.
 * This produces self-contained schemas that typify can process.
 */

import $RefParser from '@apidevtools/json-schema-ref-parser';
import { mkdir, writeFile, readFile } from 'fs/promises';
import { join, basename, dirname } from 'path';
import { existsSync } from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const SCHEMAS_DIR = join(__dirname, '5etools');
const OUTPUT_DIR = join(__dirname, 'bundled');

// Core entity schemas we want to generate types for
const SCHEMAS_TO_BUNDLE = [
  // Core entities
  'bestiary/bestiary.json',
  'items.json',
  'spells/spells.json',
  'class/class.json',
  'races.json',
  'backgrounds.json',
  'feats.json',

  // Other entities
  'actions.json',
  'conditionsdiseases.json',
  'cultsboons.json',
  'deities.json',
  'languages.json',
  'objects.json',
  'optionalfeatures.json',
  'psionics.json',
  'rewards.json',
  'senses.json',
  'skills.json',
  'tables.json',
  'trapshazards.json',
  'variantrules.json',
  'vehicles.json',

  // Shared types
  'entry.json',
  'util.json',

  // Extras
  'magicvariants.json',
];

async function bundleSchema(inputPath, outputPath) {
  try {
    // Use bundle instead of dereference - keeps circular refs as internal $refs
    // rather than trying to infinitely inline them
    const schema = await $RefParser.bundle(inputPath, {
      continueOnError: true,
      resolve: {
        external: true,
        http: false,  // Don't fetch HTTP URLs
      },
    });

    await writeFile(outputPath, JSON.stringify(schema, null, 2));
    console.log(`  ✓ ${basename(outputPath)}`);
    return true;
  } catch (err) {
    console.error(`  ✗ ${basename(inputPath)}: ${err.message}`);
    return false;
  }
}

async function main() {
  console.log('Bundling 5etools schemas...\n');
  console.log(`Input: ${SCHEMAS_DIR}`);
  console.log(`Output: ${OUTPUT_DIR}\n`);

  // Create output directory
  if (!existsSync(OUTPUT_DIR)) {
    await mkdir(OUTPUT_DIR, { recursive: true });
  }

  let success = 0;
  let failed = 0;

  for (const schemaPath of SCHEMAS_TO_BUNDLE) {
    const inputPath = join(SCHEMAS_DIR, schemaPath);
    const outputName = basename(schemaPath);
    const outputPath = join(OUTPUT_DIR, outputName);

    if (!existsSync(inputPath)) {
      console.error(`  ✗ ${schemaPath}: File not found`);
      failed++;
      continue;
    }

    if (await bundleSchema(inputPath, outputPath)) {
      success++;
    } else {
      failed++;
    }
  }

  console.log(`\nDone! Bundled ${success} schemas, ${failed} failed.`);
}

main().catch(console.error);

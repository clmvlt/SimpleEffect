import type { ItemId, SequenceEntity } from '@se/protocol';
import { ScrollArea } from '@se/ui';
import { useEffect, useRef } from 'react';

import { selectItems, setPlayheadTime } from '../../timeline';
import type { CaptionCommands } from '../hooks/use-caption-commands';
import { rowsOfTrack, type CaptionRow } from '../services/caption-rows';
import { CaptionRowItem } from './CaptionRowItem';

/** Props of {@link CaptionList}. */
export interface CaptionListProps {
  readonly sequence: SequenceEntity;
  readonly rows: readonly CaptionRow[];
  /** Index of the caption under the playhead (−1: none). */
  readonly active: number;
  readonly selected: readonly ItemId[];
  readonly commands: CaptionCommands;
}

/**
 * The captions in time order: the one under the playhead is marked and kept in view, a click
 * selects a caption and moves the playhead to it, its text is edited inline.
 */
export function CaptionList({ sequence, rows, active, selected, commands }: CaptionListProps) {
  const listRef = useRef<HTMLOListElement | null>(null);
  useEffect(() => {
    const current = listRef.current?.querySelector('[data-current]');
    if (current instanceof HTMLElement) current.scrollIntoView({ block: 'nearest' });
  }, [active]);
  return (
    <ScrollArea className="min-h-0 flex-1">
      <ol ref={listRef} className="flex flex-col gap-0.5 p-2">
        {rows.map((row, index) => {
          const neighbours = rowsOfTrack(rows, row.trackId);
          const position = neighbours.findIndex((other) => other.itemId === row.itemId);
          const previous = position > 0 ? neighbours[position - 1] : undefined;
          return (
            <CaptionRowItem
              key={row.itemId}
              row={row}
              frameRate={sequence.format.frameRate}
              current={index === active}
              selected={selected.includes(row.itemId)}
              previous={previous}
              commands={commands}
              onPick={(additive) => {
                const ids = additive
                  ? selected.includes(row.itemId)
                    ? selected.filter((id) => id !== row.itemId)
                    : [...selected, row.itemId]
                  : [row.itemId];
                selectItems(ids);
                if (!additive) setPlayheadTime(sequence.id, row.start);
              }}
            />
          );
        })}
      </ol>
    </ScrollArea>
  );
}

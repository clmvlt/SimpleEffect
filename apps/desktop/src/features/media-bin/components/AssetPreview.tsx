import type { Asset } from '@se/protocol';
import { Icon, cn } from '@se/ui';
import { AudioLines, Film, ImageIcon } from '@se/ui/icons';
import { useState } from 'react';

import { useAssetMediaUrl, useThumbnailSheet, useWaveformPeaks } from '../hooks/use-asset-media';
import { tileBackground, tileIndexAt } from '../services/thumbnail-scrub';
import { WaveformPreview } from './WaveformPreview';

/** Props of {@link AssetPreview}. */
export interface AssetPreviewProps {
  readonly asset: Asset;
  /** Width of the preview box in pixels (the height follows a 16:9 box). */
  readonly width: number;
  /** Scrub through the thumbnail sheet under the pointer (grid cards). */
  readonly scrub?: boolean;
}

/**
 * The picture of an asset in a 16:9 box: its poster, the sheet tile under the pointer while
 * hovering (videos), its waveform (sound) or a kind icon until they are generated.
 */
export function AssetPreview({ asset, width, scrub = false }: AssetPreviewProps) {
  const height = Math.round((width * 9) / 16);
  return (
    <div
      className="relative flex shrink-0 items-center justify-center overflow-hidden rounded-sm bg-bg"
      style={{ width, height }}
    >
      {asset.kind === 'audio' ? (
        <SoundPreview asset={asset} width={width} height={height} />
      ) : (
        <PicturePreview asset={asset} width={width} height={height} scrub={scrub} />
      )}
    </div>
  );
}

function SoundPreview({ asset, width, height }: { asset: Asset; width: number; height: number }) {
  const peaks = useWaveformPeaks(asset);
  return peaks === null ? (
    <Icon icon={AudioLines} size="lg" className="text-item-audio" />
  ) : (
    <WaveformPreview peaks={peaks} width={width} height={height} />
  );
}

function PicturePreview({
  asset,
  width,
  height,
  scrub,
}: {
  asset: Asset;
  width: number;
  height: number;
  scrub: boolean;
}) {
  const sheet = useThumbnailSheet(asset);
  const poster = useAssetMediaUrl(asset, 'poster', 'thumbnails');
  const sheetImage = useAssetMediaUrl(asset, 'thumbnailSheet', 'thumbnails');
  const [hovered, setHovered] = useState<number | null>(null);
  if (sheet === null || poster === null || sheetImage === null) {
    return (
      <Icon
        icon={asset.kind === 'image' ? ImageIcon : Film}
        size="lg"
        className="text-text-subtle"
      />
    );
  }
  const tile = scrub && hovered !== null && asset.kind === 'video' ? hovered : null;
  const tileWidth = Math.min(width, (height * sheet.tileWidth) / sheet.tileHeight);
  const background = tile === null ? null : tileBackground(sheet, tile, tileWidth);
  return (
    <div
      className="flex size-full items-center justify-center"
      onPointerMove={(event) => {
        const box = event.currentTarget.getBoundingClientRect();
        const fraction = box.width > 0 ? (event.clientX - box.left) / box.width : 0;
        setHovered(tileIndexAt(fraction, sheet.times.length));
      }}
      onPointerLeave={() => {
        setHovered(null);
      }}
    >
      {background === null ? (
        <img src={poster} alt="" draggable={false} className={cn('max-h-full max-w-full')} />
      ) : (
        <div
          aria-hidden
          style={{
            width: tileWidth,
            height: background.height,
            backgroundImage: `url("${sheetImage}")`,
            backgroundSize: background.size,
            backgroundPosition: background.position,
          }}
        />
      )}
    </div>
  );
}

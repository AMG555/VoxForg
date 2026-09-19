import React, { useState } from 'react';
import { Trash2, Palette, StickyNote } from 'lucide-react';
import { StickyNoteData } from '../../services/workflowStorage';

interface StickyNoteCardProps {
  note: StickyNoteData;
  onUpdate: (note: StickyNoteData) => void;
  onDelete: (id: string) => void;
  onPointerDown?: (e: React.PointerEvent, id: string) => void;
  style?: React.CSSProperties;
}

const colorStyles = {
  amber: {
    card: 'bg-amber-950/30 border-amber-500/40 text-amber-200',
    header: 'bg-amber-900/30 border-amber-500/30 text-amber-300',
    accent: 'text-amber-400',
  },
  sky: {
    card: 'bg-sky-950/30 border-sky-500/40 text-sky-200',
    header: 'bg-sky-900/30 border-sky-500/30 text-sky-300',
    accent: 'text-sky-400',
  },
  emerald: {
    card: 'bg-emerald-950/30 border-emerald-500/40 text-emerald-200',
    header: 'bg-emerald-900/30 border-emerald-500/30 text-emerald-300',
    accent: 'text-emerald-400',
  },
  purple: {
    card: 'bg-purple-950/30 border-purple-500/40 text-purple-200',
    header: 'bg-purple-900/30 border-purple-500/30 text-purple-300',
    accent: 'text-purple-400',
  },
  rose: {
    card: 'bg-rose-950/30 border-rose-500/40 text-rose-200',
    header: 'bg-rose-900/30 border-rose-500/30 text-rose-300',
    accent: 'text-rose-400',
  },
};

export const StickyNoteCard: React.FC<StickyNoteCardProps> = ({
  note,
  onUpdate,
  onDelete,
  onPointerDown,
  style,
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const color = colorStyles[note.color] || colorStyles.amber;

  const handleNextColor = (e: React.MouseEvent) => {
    e.stopPropagation();
    const colors: Array<StickyNoteData['color']> = ['amber', 'sky', 'emerald', 'purple', 'rose'];
    const curIdx = colors.indexOf(note.color);
    const nextColor = colors[(curIdx + 1) % colors.length];
    onUpdate({ ...note, color: nextColor });
  };

  return (
    <div
      onPointerDown={(e) => onPointerDown?.(e, note.id)}
      className={`absolute rounded-xl border backdrop-blur-md shadow-xl select-none cursor-grab active:cursor-grabbing transition-all ${color.card}`}
      style={{
        ...style,
        width: `${note.width || 260}px`,
      }}
    >
      {/* Header */}
      <div
        className={`px-3 py-2 border-b rounded-t-xl flex items-center justify-between ${color.header}`}
      >
        <div className="flex items-center space-x-1.5 min-w-0">
          <StickyNote className={`w-3.5 h-3.5 ${color.accent}`} />
          {isEditing ? (
            <input
              type="text"
              value={note.title}
              onChange={(e) => onUpdate({ ...note, title: e.target.value })}
              onBlur={() => setIsEditing(false)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') setIsEditing(false);
              }}
              autoFocus
              className="bg-black/40 border border-white/20 rounded px-1.5 py-0.5 text-xs text-white font-mono focus:outline-none w-36"
            />
          ) : (
            <span
              onClick={() => setIsEditing(true)}
              className="text-xs font-semibold font-mono truncate cursor-pointer hover:underline"
            >
              {note.title || 'Note'}
            </span>
          )}
        </div>

        <div className="flex items-center space-x-1">
          <button
            type="button"
            onClick={handleNextColor}
            className="p-1 rounded hover:bg-white/10 text-white/70 hover:text-white transition-colors"
            title="Cycle note color"
          >
            <Palette className="w-3 h-3" />
          </button>
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onDelete(note.id);
            }}
            className="p-1 rounded hover:bg-rose-500/20 text-white/60 hover:text-rose-400 transition-colors"
            title="Delete note"
          >
            <Trash2 className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Body */}
      <div className="p-3">
        <textarea
          rows={3}
          value={note.content}
          onChange={(e) => onUpdate({ ...note, content: e.target.value })}
          placeholder="Write documentation, stage notes, or comments..."
          className="w-full bg-transparent border-0 text-xs font-sans text-white/90 placeholder-white/40 focus:outline-none resize-none leading-relaxed"
        />
      </div>
    </div>
  );
};

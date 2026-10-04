import React, { useState, useEffect } from 'react';
import { X, BookA, Plus, Trash2, CheckCircle2, Sparkles } from 'lucide-react';
import { api } from '../../services/api';

interface PronunciationModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface DictionaryEntry {
  term: string;
  replacement: string;
  note?: string;
}

export const PronunciationModal: React.FC<PronunciationModalProps> = ({ isOpen, onClose }) => {
  const [entries, setEntries] = useState<DictionaryEntry[]>([]);
  const [term, setTerm] = useState('');
  const [replacement, setReplacement] = useState('');
  const [note, setNote] = useState('');
  const [testText, setTestText] = useState('Run this SQL query etc.');
  const [testResult, setTestResult] = useState('');
  const [saving, setSaving] = useState(false);
  const [statusMsg, setStatusMsg] = useState('');

  const loadEntries = async () => {
    try {
      const res = await api.getDictionary();
      setEntries(res.entries || []);
    } catch {
      // Fallback local defaults
      setEntries([
        { term: 'SQL', replacement: 'Sequel', note: 'Standard database query language' },
        { term: 'etc.', replacement: 'et cetera', note: 'Latin abbreviation' },
        { term: 'Dr.', replacement: 'Doctor', note: 'Title abbreviation' },
      ]);
    }
  };

  useEffect(() => {
    if (isOpen) {
      loadEntries();
    }
  }, [isOpen]);

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!term.trim() || !replacement.trim()) return;

    try {
      setSaving(true);
      const newEntry = { term: term.trim(), replacement: replacement.trim(), note: note.trim() || undefined };
      await api.upsertDictionary(newEntry);
      setEntries((prev) => [newEntry, ...prev.filter((item) => item.term !== newEntry.term)]);
      setTerm('');
      setReplacement('');
      setNote('');
      setStatusMsg(`Added "${newEntry.term}"`);
      setTimeout(() => setStatusMsg(''), 2500);
    } catch (err: any) {
      alert(`Failed to save entry: ${err.message}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (targetTerm: string) => {
    try {
      await api.deleteDictionary(targetTerm);
      setEntries((prev) => prev.filter((item) => item.term !== targetTerm));
    } catch (err: any) {
      alert(`Failed to delete entry: ${err.message}`);
    }
  };

  const handleTestApply = async () => {
    if (!testText.trim()) return;
    try {
      const res = await api.applyPronunciation(testText);
      setTestResult(res.processed);
    } catch {
      let simulated = testText;
      for (const item of entries) {
        const re = new RegExp(`\\b${item.term}\\b`, 'gi');
        simulated = simulated.replace(re, item.replacement);
      }
      setTestResult(simulated);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-[#121820] border border-[#242E3D] rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col font-mono text-xs overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="px-6 py-4 border-b border-[#242E3D] bg-[#0E131A] flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-400">
              <BookA className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-white tracking-tight flex items-center space-x-2">
                <span>Pronunciation Lexicon</span>
                <span className="px-2 py-0.5 rounded-full text-[10px] bg-amber-500/20 text-amber-300 border border-amber-500/40">
                  {entries.length} Entries
                </span>
              </h2>
              <p className="text-[11px] text-[#94A3B8] font-sans">
                Whole-word boundary regex substitutions and phonetic respellings across all speech engines
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-[#94A3B8] hover:text-white hover:bg-[#1A222D] transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content Body */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Add Entry Form */}
          <form onSubmit={handleAdd} className="p-4 rounded-xl bg-[#0B0E14] border border-[#242E3D] space-y-3">
            <div className="text-[11px] uppercase tracking-wider text-amber-400 font-bold flex items-center justify-between">
              <span>Add Pronunciation Rule</span>
              {statusMsg && <span className="text-emerald-400 normal-case">{statusMsg}</span>}
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label className="text-[10px] text-[#94A3B8] block mb-1">Original Term / Word</label>
                <input
                  type="text"
                  value={term}
                  onChange={(e) => setTerm(e.target.value)}
                  placeholder="e.g. SQL, Dr., epazote"
                  className="w-full bg-[#121820] border border-[#242E3D] rounded-lg px-3 py-1.5 text-white placeholder-[#64748B] focus:border-amber-500 focus:outline-none"
                  required
                />
              </div>
              <div>
                <label className="text-[10px] text-[#94A3B8] block mb-1">Pronounce As (Respelling)</label>
                <input
                  type="text"
                  value={replacement}
                  onChange={(e) => setReplacement(e.target.value)}
                  placeholder="e.g. Sequel, Doctor"
                  className="w-full bg-[#121820] border border-[#242E3D] rounded-lg px-3 py-1.5 text-white placeholder-[#64748B] focus:border-amber-500 focus:outline-none"
                  required
                />
              </div>
            </div>
            <div>
              <label className="text-[10px] text-[#94A3B8] block mb-1">Optional Note / Language Scope</label>
              <div className="flex items-center space-x-2">
                <input
                  type="text"
                  value={note}
                  onChange={(e) => setNote(e.target.value)}
                  placeholder="e.g. Acronym / medical term"
                  className="flex-1 bg-[#121820] border border-[#242E3D] rounded-lg px-3 py-1.5 text-white placeholder-[#64748B] focus:border-amber-500 focus:outline-none"
                />
                <button
                  type="submit"
                  disabled={saving || !term.trim() || !replacement.trim()}
                  className="px-4 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 disabled:opacity-50 text-black font-bold flex items-center space-x-1.5 transition-colors shrink-0"
                >
                  <Plus className="w-3.5 h-3.5 stroke-[2.5]" />
                  <span>Save Rule</span>
                </button>
              </div>
            </div>
          </form>

          {/* Live Tester */}
          <div className="p-4 rounded-xl bg-[#0B0E14] border border-[#242E3D] space-y-3">
            <div className="flex items-center justify-between text-[11px] uppercase tracking-wider text-sky-400 font-bold">
              <span>Interactive Rule Tester</span>
              <button
                type="button"
                onClick={handleTestApply}
                className="flex items-center space-x-1 text-[10px] text-amber-400 hover:text-amber-300 normal-case"
              >
                <Sparkles className="w-3.5 h-3.5" />
                <span>Simulate Normalization</span>
              </button>
            </div>
            <input
              type="text"
              value={testText}
              onChange={(e) => setTestText(e.target.value)}
              placeholder="Type test sentence here..."
              className="w-full bg-[#121820] border border-[#242E3D] rounded-lg px-3 py-2 text-white placeholder-[#64748B] focus:border-sky-500 focus:outline-none"
            />
            {testResult && (
              <div className="p-2.5 rounded-lg bg-[#161F2C] border border-sky-800/40 text-sky-300 text-xs flex items-start space-x-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <div>
                  <span className="text-[10px] text-[#94A3B8] block">Normalized Output for TTS:</span>
                  <span className="font-semibold text-white">{testResult}</span>
                </div>
              </div>
            )}
          </div>

          {/* Lexicon Table */}
          <div className="space-y-2">
            <span className="text-[11px] uppercase tracking-wider text-[#94A3B8] block">
              Active Dictionary Entries ({entries.length})
            </span>
            <div className="border border-[#242E3D] rounded-xl overflow-hidden bg-[#0B0E14] divide-y divide-[#242E3D]/60 max-h-56 overflow-y-auto">
              {entries.length === 0 ? (
                <div className="p-6 text-center text-[#64748B]">No dictionary entries found.</div>
              ) : (
                entries.map((item) => (
                  <div
                    key={item.term}
                    className="p-3 flex items-center justify-between hover:bg-[#161F2C]/40 transition-colors"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center space-x-2">
                        <span className="font-bold text-white">{item.term}</span>
                        <span className="text-[#64748B]">→</span>
                        <span className="text-amber-400 font-semibold">{item.replacement}</span>
                      </div>
                      {item.note && <p className="text-[10px] text-[#64748B] truncate mt-0.5">{item.note}</p>}
                    </div>
                    <button
                      onClick={() => handleDelete(item.term)}
                      className="p-1 rounded text-[#64748B] hover:text-rose-400 hover:bg-rose-500/10 transition-colors ml-2"
                      title="Delete rule"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-[#242E3D] bg-[#0E131A] flex items-center justify-end">
          <button
            onClick={onClose}
            className="px-4 py-1.5 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-white transition-colors"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
};

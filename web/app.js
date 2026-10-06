// Draws what the core sends and passes what the user types back to it.
// Keeps no rule: sets, swaps, codes, files, checks and every message come from
// the core (crates/core) through the shim (pkg/warmups_wasm.js).
// No template strings or regular expressions here: the boundary test reads
// this file and expects plain quotes.
import init, { options, make_set, swap, open_set, check, set_file } from './pkg/warmups_wasm.js';

const el = (id) => document.getElementById(id);
const state = { bank: '', teacherCode: '', studentSet: null };

function make(tag, attrs, text) {
  const node = document.createElement(tag);
  for (const [name, value] of Object.entries(attrs || {})) {
    node.setAttribute(name, value);
  }
  if (text !== undefined) node.textContent = text;
  return node;
}

function showMessage(id, reply) {
  const box = el(id);
  box.replaceChildren();
  if (reply.ok) return;
  box.append(make('p', {}, reply.error));
  if (reply.problems) {
    const list = make('ul');
    reply.problems.forEach((p) => list.append(make('li', {}, p)));
    box.append(list);
  }
}

function drawChoices(containerId, items, type, name) {
  const box = el(containerId);
  box.replaceChildren();
  items.forEach((item) => {
    const input = make('input', { type: type, name: name, value: item.value });
    if (item.default) input.checked = true;
    const label = make('label', {});
    label.append(input, ' ' + item.label);
    box.append(label);
  });
}

function drawSources(set, box) {
  set.sources.forEach((s) => {
    const card = make('div', { class: 'source' });
    card.append(make('h3', {}, 'Source ' + s.id + ': ' + s.title), make('p', {}, s.text));
    box.append(card);
  });
}

function drawSet(boxId, set) {
  const box = el(boxId);
  box.replaceChildren();
  box.append(make('h2', {}, set.title));
  box.append(make('p', { class: 'code' }, 'Set code: ' + set.code));
  box.append(make('p', { class: 'notice small' }, set.notice));
  drawSources(set, box);
  const list = make('ol', { class: 'questions' });
  set.questions.forEach((q) => list.append(drawQuestion(set, q)));
  box.append(list);
  if (set.swap_note) box.append(make('p', { class: 'swap-note no-print' }, set.swap_note));
}

function drawQuestion(set, q) {
  const item = make('li', { class: 'question', 'data-id': q.id, 'data-number': q.number });
  item.append(make('p', { class: 'prompt' }, q.prompt));
  const name = 'answer-' + q.number;
  if (set.teacher) {
    q.options.forEach((o) => item.append(make('p', { class: 'option' }, o.letter + ') ' + o.text)));
  } else if (q.input === 'choice') {
    q.options.forEach((o) => {
      const label = make('label', { class: 'option' });
      label.append(make('input', { type: 'radio', name: name, value: o.letter }), ' ' + o.letter + ') ' + o.text);
      item.append(label);
    });
  } else {
    const label = make('label', { class: 'answer' }, 'Your answer ');
    label.append(make('input', { type: 'text', name: name, autocomplete: 'off', spellcheck: 'false' }));
    item.append(label);
  }
  item.append(make('p', { class: 'hint' }, q.hint));
  if (set.teacher) {
    item.append(make('p', { class: 'answer-line print-only' }, 'Answer: ______________________'));
    item.append(make('p', { class: 'meta no-print' }, 'Source ' + q.source + ', DOK ' + q.level + ', question ' + q.id));
    item.append(make('p', { class: 'key no-print' }, 'Key: ' + q.key));
    if (q.note) item.append(make('p', { class: 'note no-print' }, q.note));
    const button = make('button', { type: 'button', class: 'swap no-print' }, 'Swap question ' + q.number);
    button.disabled = !q.can_swap;
    button.addEventListener('click', () => onSwap(q.number));
    item.append(button);
  } else {
    item.append(make('p', { class: 'mark', 'aria-live': 'polite' }));
  }
  return item;
}

function showTeacherReply(reply) {
  showMessage('teacher-message', reply);
  if (!reply.ok) return;
  state.teacherCode = reply.set.code;
  drawSet('teacher-set', reply.set);
  el('teacher-actions').hidden = false;
}

function onMake(event) {
  event.preventDefault();
  const form = el('make-form');
  const sources = Array.from(form.querySelectorAll('input[name=source]:checked')).map((i) => i.value);
  const level = form.querySelector('input[name=level]:checked');
  const kind = form.querySelector('input[name=kind]:checked');
  const request = { kind: kind ? kind.value : '', sources: sources, level: level ? level.value : '' };
  const reply = JSON.parse(make_set(state.bank, JSON.stringify(request)));
  if (!reply.ok) {
    el('teacher-set').replaceChildren();
    el('teacher-actions').hidden = true;
  }
  showTeacherReply(reply);
}

function onSwap(number) {
  const reply = JSON.parse(swap(state.bank, state.teacherCode, number));
  showTeacherReply(reply);
}

function onSave() {
  const reply = JSON.parse(set_file(state.bank, state.teacherCode));
  showMessage('teacher-message', reply);
  if (!reply.ok) return;
  const url = URL.createObjectURL(new Blob([reply.text], { type: 'text/plain' }));
  const link = make('a', { href: url, download: reply.file_name });
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

function openStudentSet(text) {
  const reply = JSON.parse(open_set(state.bank, text));
  showMessage('student-message', reply);
  el('summary').textContent = '';
  if (!reply.ok) {
    state.studentSet = null;
    el('student-set').replaceChildren();
    el('check').hidden = true;
    return;
  }
  state.studentSet = reply.set;
  drawSet('student-set', reply.set);
  el('check').hidden = false;
}

function onOpen(event) {
  event.preventDefault();
  openStudentSet(el('code-input').value);
}

function onFile() {
  const file = el('file-input').files[0];
  if (file) file.text().then(openStudentSet);
}

function onCheck(event) {
  event.preventDefault();
  const set = state.studentSet;
  if (!set) return;
  const form = el('answer-form');
  const answers = set.questions.map((q) => {
    const name = 'answer-' + q.number;
    const field = form.querySelector('input[name="' + name + '"]:checked') || form.querySelector('input[type=text][name="' + name + '"]');
    return field ? field.value : '';
  });
  const reply = JSON.parse(check(state.bank, set.code, JSON.stringify(answers)));
  showMessage('student-message', reply);
  if (!reply.ok) return;
  reply.result.marks.forEach((m) => {
    const mark = form.querySelector('li[data-number="' + m.number + '"] .mark');
    mark.textContent = m.label;
    mark.className = 'mark ' + (m.matches ? 'match' : 'no-match');
  });
  el('summary').textContent = reply.result.summary;
}

function setMode(student) {
  el('teacher').hidden = student;
  el('student').hidden = !student;
  el('mode-teacher').setAttribute('aria-pressed', String(!student));
  el('mode-student').setAttribute('aria-pressed', String(student));
}

async function start() {
  await init();
  const response = await fetch('data/bank.json');
  state.bank = await response.text();
  const reply = JSON.parse(options(state.bank));
  showMessage('teacher-message', reply);
  if (!reply.ok) return;
  el('notice').textContent = reply.notice;
  drawChoices('sources', reply.sources.map((s) => ({ value: s.id, label: s.id + ': ' + s.title })), 'checkbox', 'source');
  drawChoices('levels', reply.levels, 'radio', 'level');
  drawChoices('kinds', reply.kinds, 'radio', 'kind');
  el('make-form').addEventListener('submit', onMake);
  el('print').addEventListener('click', () => window.print());
  el('save').addEventListener('click', onSave);
  el('open-form').addEventListener('submit', onOpen);
  el('file-input').addEventListener('change', onFile);
  el('answer-form').addEventListener('submit', onCheck);
  el('mode-teacher').addEventListener('click', () => setMode(false));
  el('mode-student').addEventListener('click', () => setMode(true));
  document.body.dataset.ready = 'yes';
}

start().catch((err) => {
  el('teacher-message').textContent = 'The tool could not start: ' + err;
  console.error(err);
});

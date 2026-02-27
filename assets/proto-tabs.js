function showState(id) {
  document.querySelectorAll('.proto-state').forEach(function(el) {
    el.style.display = 'none';
  });
  var target = document.getElementById(id);
  if (target) target.style.display = 'flex';
  document.querySelectorAll('.proto-tab').forEach(function(el) {
    el.classList.remove('bg-blue-600', 'text-white');
    el.classList.add('bg-gray-200', 'text-gray-700');
  });
  var btn = document.querySelector('[data-tab="' + id + '"]');
  if (btn) {
    btn.classList.add('bg-blue-600', 'text-white');
    btn.classList.remove('bg-gray-200', 'text-gray-700');
  }
}

document.addEventListener('DOMContentLoaded', function() {
  var first = document.querySelector('.proto-state');
  if (first) showState(first.id);
});

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal guild-settings-modal">
      <!-- Сайдбар -->
      <div class="settings-sidebar">
        <div class="sidebar-header">{{ guild?.name }}</div>
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="settings-tab"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </div>
      </div>

      <!-- Контент -->
      <div class="settings-content">
        <button class="settings-close" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>

        <!-- === УЧАСТНИКИ === -->
        <template v-if="activeTab === 'members'">
          <h2>Участники</h2>
          <div class="settings-divider" />
          <input v-model="memberSearch" class="search-input" placeholder="Поиск по имени..." />
          <div class="member-list">
            <div v-for="m in filteredMembers" :key="m.user_id" class="member-row">
              <div class="member-avatar">{{ m.username[0]?.toUpperCase() }}</div>
              <div class="member-info">
                <span class="member-name">{{ m.nickname || m.username }}</span>
                <span v-if="m.is_muted" class="member-badge badge-muted">заглушён</span>
                <span v-if="isTimedOut(m)" class="member-badge badge-timeout">таймаут</span>
                <div class="member-roles">
                  <span
                    v-for="rid in m.role_ids"
                    :key="rid"
                    class="role-chip"
                    :style="{ borderColor: roleColor(rid) }"
                  >{{ roleName(rid) }}</span>
                </div>
              </div>
              <div v-if="m.user_id !== authUserId && m.user_id !== guild?.owner_id" class="member-actions">
                <button
                  v-if="guildStore.hasPermission(PERM.MUTE_MEMBERS)"
                  class="btn-icon"
                  :title="m.is_muted ? 'Снять глушение' : 'Заглушить'"
                  @click="toggleMute(m)"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path v-if="m.is_muted" d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/>
                    <path v-else d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z"/>
                  </svg>
                </button>
                <button
                  v-if="guildStore.hasPermission(PERM.MUTE_MEMBERS)"
                  class="btn-icon"
                  title="Таймаут (10 мин)"
                  @click="applyTimeout(m)"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67V7z"/>
                  </svg>
                </button>
                <button
                  v-if="guildStore.hasPermission(PERM.MANAGE_MEMBERS)"
                  class="btn-icon btn-icon-danger"
                  title="Кикнуть"
                  @click="kick(m)"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M13 3h-2v10h2V3zm4.83 2.17l-1.42 1.42C17.99 7.86 19 9.81 19 12c0 3.87-3.13 7-7 7s-7-3.13-7-7c0-2.19 1.01-4.14 2.58-5.42L6.17 5.17C4.23 6.82 3 9.26 3 12c0 4.97 4.03 9 9 9s9-4.03 9-9c0-2.74-1.23-5.18-3.17-6.83z"/>
                  </svg>
                </button>
                <button
                  v-if="guildStore.hasPermission(PERM.BAN_MEMBERS)"
                  class="btn-icon btn-icon-danger"
                  title="Забанить"
                  @click="ban(m)"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 15H9V8h2v9zm4 0h-2V8h2v9z"/>
                  </svg>
                </button>
              </div>
            </div>
            <div v-if="filteredMembers.length === 0" class="empty-state">Нет участников</div>
          </div>
        </template>

        <!-- === РОЛИ === -->
        <template v-if="activeTab === 'roles'">
          <h2>Роли</h2>
          <div class="settings-divider" />
          <div class="roles-layout">
            <!-- Список ролей -->
            <div class="roles-list">
              <div
                v-for="role in guildStore.roles"
                :key="role.id"
                class="role-row"
                :class="{ active: selectedRoleId === role.id }"
                @click="selectRole(role.id)"
              >
                <span class="role-dot" :style="{ background: role.color || '#8a8a8a' }"></span>
                <span class="role-row-name">{{ role.name }}</span>
              </div>
              <button
                v-if="guildStore.hasPermission(PERM.MANAGE_ROLES)"
                class="btn-secondary btn-sm role-add-btn"
                @click="createNewRole"
              >+ Создать роль</button>
            </div>

            <!-- Редактор выбранной роли -->
            <div v-if="selectedRole" class="role-editor">
              <div class="role-editor-header">
                <input v-model="editName" class="role-name-input" placeholder="Название роли"
                  :disabled="selectedRole.name === '@everyone'" />
                <input v-model="editColor" type="color" class="role-color-input"
                  :value="editColor || '#8a8a8a'" />
              </div>

              <div class="perm-section">
                <div class="perm-label">Права</div>
                <label
                  v-for="(bit, key) in PERM"
                  :key="key"
                  class="perm-row"
                  :class="{ 'perm-disabled': key !== 'ADMINISTRATOR' && hasAdminPerm }"
                >
                  <input
                    type="checkbox"
                    :checked="hasPermBit(bit)"
                    :disabled="key !== 'ADMINISTRATOR' && hasAdminPerm && selectedRole.name !== '@everyone'"
                    @change="togglePermBit(bit)"
                  />
                  <span>{{ PERM_LABELS[key] }}</span>
                </label>
              </div>

              <div v-if="guildStore.hasPermission(PERM.MANAGE_ROLES)" class="role-editor-actions">
                <button class="btn-primary btn-sm" @click="saveRole">Сохранить</button>
                <button
                  v-if="selectedRole.name !== '@everyone'"
                  class="btn-danger btn-sm"
                  @click="deleteSelectedRole"
                >Удалить роль</button>
              </div>

              <!-- Назначение роли участникам -->
              <div v-if="selectedRole.name !== '@everyone' && guildStore.hasPermission(PERM.MANAGE_ROLES)" class="role-assign-section">
                <div class="perm-label">Участники с этой ролью</div>
                <div class="role-members">
                  <div
                    v-for="m in membersWithRole(selectedRoleId!)"
                    :key="m.user_id"
                    class="role-member-row"
                  >
                    <span>{{ m.nickname || m.username }}</span>
                    <button class="btn-icon btn-icon-danger" @click="removeRoleFromMember(m.user_id)" title="Снять роль">×</button>
                  </div>
                </div>
                <div class="role-assign-add">
                  <select v-model="assignTarget" class="assign-select">
                    <option value="">Назначить участнику...</option>
                    <option
                      v-for="m in membersWithoutRole(selectedRoleId!)"
                      :key="m.user_id"
                      :value="m.user_id"
                    >{{ m.nickname || m.username }}</option>
                  </select>
                  <button class="btn-secondary btn-sm" :disabled="!assignTarget" @click="assignRoleToMember">Назначить</button>
                </div>
              </div>
            </div>
            <div v-else class="role-editor role-editor-empty">Выберите роль</div>
          </div>
        </template>

        <!-- === ИНВАЙТЫ === -->
        <template v-if="activeTab === 'invites'">
          <h2>Приглашения</h2>
          <div class="settings-divider" />

          <div class="invite-create">
            <div class="invite-create-fields">
              <div class="invite-field">
                <label>Максимум использований (0 = без лимита)</label>
                <input v-model.number="newMaxUses" type="number" min="0" class="invite-input" />
              </div>
              <div class="invite-field">
                <label>Срок действия (часов, 0 = бессрочно)</label>
                <input v-model.number="newExpiresHours" type="number" min="0" class="invite-input" />
              </div>
            </div>
            <button class="btn-primary btn-sm" @click="createInvite">Создать инвайт</button>
          </div>

          <div class="settings-divider" />

          <div class="invite-list">
            <div v-for="inv in guildStore.invites" :key="inv.code" class="invite-row">
              <div class="invite-info">
                <span class="invite-code">{{ inv.code }}</span>
                <span class="invite-meta">{{ inv.uses }}{{ inv.max_uses ? '/' + inv.max_uses : '' }} использований</span>
                <span v-if="inv.expires_at" class="invite-meta">до {{ formatDate(inv.expires_at) }}</span>
              </div>
              <div class="invite-actions">
                <button class="btn-icon" title="Копировать" @click="copyInvite(inv.code)">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                  </svg>
                </button>
                <button class="btn-icon btn-icon-danger" title="Отозвать" @click="revokeInvite(inv.code)">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                  </svg>
                </button>
              </div>
            </div>
            <div v-if="guildStore.invites.length === 0" class="empty-state">Нет активных инвайтов</div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useGuildStore, PERM, type Member } from '../../stores/guild'
import { useAuthStore } from '../../stores/auth'

const props = defineProps<{
  guildId: string
}>()
defineEmits(['close'])

const guildStore = useGuildStore()
const auth = useAuthStore()

const authUserId = computed(() => auth.userId)
const guild = computed(() => guildStore.guilds.find(g => g.id === props.guildId))

const PERM_LABELS: Record<string, string> = {
  ADMINISTRATOR:   'Администратор',
  MANAGE_CHANNELS: 'Управление каналами',
  MANAGE_ROLES:    'Управление ролями',
  MANAGE_MEMBERS:  'Управление участниками',
  SEND_MESSAGES:   'Отправка сообщений',
  ATTACH_FILES:    'Прикрепление файлов',
  CONNECT_VOICE:   'Подключение к голосу',
  STREAM_SCREEN:   'Трансляция экрана',
  MUTE_MEMBERS:    'Заглушение участников',
  BAN_MEMBERS:     'Бан участников',
}

// Вкладки зависят от прав
const tabs = computed(() => {
  const list = []
  if (guildStore.hasPermission(PERM.MANAGE_MEMBERS) || guildStore.hasPermission(PERM.MUTE_MEMBERS) || guildStore.hasPermission(PERM.BAN_MEMBERS)) {
    list.push({ id: 'members', label: 'Участники' })
  }
  if (guildStore.hasPermission(PERM.MANAGE_ROLES)) {
    list.push({ id: 'roles', label: 'Роли' })
  }
  if (guildStore.hasPermission(PERM.MANAGE_MEMBERS)) {
    list.push({ id: 'invites', label: 'Инвайты' })
  }
  return list
})

const activeTab = ref(tabs.value[0]?.id ?? 'members')

// === УЧАСТНИКИ ===
const memberSearch = ref('')
const filteredMembers = computed(() =>
  guildStore.members.filter(m =>
    (m.nickname || m.username).toLowerCase().includes(memberSearch.value.toLowerCase())
  )
)

function isTimedOut(m: Member) {
  return m.timeout_until && new Date(m.timeout_until) > new Date()
}

function roleName(rid: string) {
  return guildStore.roles.find(r => r.id === rid)?.name ?? rid.slice(0, 8)
}

function roleColor(rid: string) {
  return guildStore.roles.find(r => r.id === rid)?.color ?? '#555'
}

async function toggleMute(m: Member) {
  await guildStore.muteMember(props.guildId, m.user_id, !m.is_muted)
}

async function applyTimeout(m: Member) {
  // 10 минут
  await guildStore.timeoutMember(props.guildId, m.user_id, 600)
}

async function kick(m: Member) {
  if (confirm(`Кикнуть ${m.nickname || m.username}?`)) {
    await guildStore.kickMember(props.guildId, m.user_id)
  }
}

async function ban(m: Member) {
  if (confirm(`Забанить ${m.nickname || m.username}? Это необратимо.`)) {
    await guildStore.banMember(props.guildId, m.user_id)
  }
}

// === РОЛИ ===
const selectedRoleId = ref<string | null>(null)
const editName = ref('')
const editColor = ref('')
const editPerms = ref(0)
const assignTarget = ref('')

const selectedRole = computed(() =>
  selectedRoleId.value ? guildStore.roles.find(r => r.id === selectedRoleId.value) : null
)

const hasAdminPerm = computed(() => (editPerms.value & PERM.ADMINISTRATOR) !== 0)

function selectRole(id: string) {
  selectedRoleId.value = id
  assignTarget.value = ''
  const r = guildStore.roles.find(r => r.id === id)
  if (r) {
    editName.value = r.name
    editColor.value = r.color ?? ''
    editPerms.value = r.permissions
  }
}

function hasPermBit(bit: number) {
  if (hasAdminPerm.value && bit !== PERM.ADMINISTRATOR) return true
  return (editPerms.value & bit) !== 0
}

function togglePermBit(bit: number) {
  editPerms.value ^= bit
  // ADMINISTRATOR включает/выключает всё
  if (bit === PERM.ADMINISTRATOR) {
    if (editPerms.value & PERM.ADMINISTRATOR) {
      editPerms.value = Object.values(PERM).reduce((a, b) => a | b, 0)
    }
  }
}

async function createNewRole() {
  const r = await guildStore.createRole(props.guildId, { name: 'Новая роль', permissions: 0 })
  selectRole(r.id)
}

async function saveRole() {
  if (!selectedRoleId.value) return
  await guildStore.updateRole(props.guildId, selectedRoleId.value, {
    name: editName.value,
    color: editColor.value || undefined,
    permissions: editPerms.value,
  })
}

async function deleteSelectedRole() {
  if (!selectedRoleId.value) return
  if (!confirm('Удалить роль?')) return
  await guildStore.deleteRole(props.guildId, selectedRoleId.value)
  selectedRoleId.value = null
}

function membersWithRole(roleId: string) {
  return guildStore.members.filter(m => m.role_ids.includes(roleId))
}

function membersWithoutRole(roleId: string) {
  return guildStore.members.filter(m => !m.role_ids.includes(roleId))
}

async function assignRoleToMember() {
  if (!assignTarget.value || !selectedRoleId.value) return
  await guildStore.assignRole(props.guildId, assignTarget.value, selectedRoleId.value)
  assignTarget.value = ''
}

async function removeRoleFromMember(userId: string) {
  if (!selectedRoleId.value) return
  await guildStore.removeRole(props.guildId, userId, selectedRoleId.value)
}

// === ИНВАЙТЫ ===
const newMaxUses = ref(0)
const newExpiresHours = ref(0)

async function createInvite() {
  await guildStore.createInviteWithOptions(props.guildId, {
    maxUses: newMaxUses.value > 0 ? newMaxUses.value : undefined,
    expiresHours: newExpiresHours.value > 0 ? newExpiresHours.value : undefined,
  })
}

async function revokeInvite(code: string) {
  await guildStore.deleteInvite(props.guildId, code)
}

function copyInvite(code: string) {
  navigator.clipboard.writeText(code)
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('ru-RU', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })
}

// Загрузка данных при открытии
onMounted(async () => {
  await Promise.all([
    guildStore.loadRoles(props.guildId),
    guildStore.loadMembers(props.guildId),
    guildStore.hasPermission(PERM.MANAGE_MEMBERS) ? guildStore.loadInvites(props.guildId) : Promise.resolve(),
  ])
  await guildStore.loadMyPermissions(props.guildId)
  if (tabs.value.length > 0) activeTab.value = tabs.value[0].id
})

watch(() => props.guildId, async (id) => {
  if (!id) return
  await Promise.all([
    guildStore.loadRoles(id),
    guildStore.loadMembers(id),
  ])
})
</script>

<style scoped>
.guild-settings-modal {
  display: flex;
  width: 780px;
  max-width: 95vw;
  height: 560px;
  max-height: 90vh;
  overflow: hidden;
  padding: 0;
}

.settings-sidebar {
  width: 180px;
  min-width: 180px;
  background: var(--bg-base);
  border-right: 1px solid var(--border);
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-header {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
  padding: 4px 8px 8px;
}

.settings-tab {
  padding: 7px 10px;
  border-radius: 6px;
  font-size: 14px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.settings-tab:hover { background: var(--bg-hover); color: var(--text-primary); }
.settings-tab.active { background: var(--bg-active); color: var(--text-primary); }

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  position: relative;
}

.settings-close {
  position: absolute;
  top: 16px;
  right: 16px;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
}
.settings-close:hover { background: var(--bg-active); color: var(--text-primary); }

h2 {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 4px;
}

.settings-divider {
  height: 1px;
  background: var(--border);
  margin: 12px 0;
}

/* Поиск */
.search-input {
  width: 100%;
  padding: 7px 10px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  margin-bottom: 12px;
}

/* Список участников */
.member-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.member-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-radius: 6px;
  transition: background 0.12s;
}
.member-row:hover { background: var(--bg-hover); }

.member-avatar {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--bg-active);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.member-info {
  flex: 1;
  min-width: 0;
}

.member-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.member-badge {
  font-size: 11px;
  padding: 1px 5px;
  border-radius: 4px;
  margin-left: 6px;
}
.badge-muted { background: #4a2; color: #fff; }
.badge-timeout { background: #e67e22; color: #fff; }

.member-roles {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 3px;
}

.role-chip {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid;
  color: var(--text-secondary);
}

.member-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* Кнопки иконок */
.btn-icon {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.btn-icon:hover { background: var(--bg-active); color: var(--text-primary); }
.btn-icon-danger:hover { background: rgba(230, 57, 70, 0.15); color: #e63946; }

/* Роли */
.roles-layout {
  display: flex;
  gap: 16px;
  height: calc(100% - 60px);
}

.roles-list {
  width: 160px;
  min-width: 160px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.role-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}
.role-row:hover { background: var(--bg-hover); }
.role-row.active { background: var(--bg-active); }

.role-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.role-row-name {
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.role-add-btn {
  margin-top: 8px;
}

.role-editor {
  flex: 1;
  overflow-y: auto;
}

.role-editor-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 14px;
}

.role-editor-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 14px;
}

.role-name-input {
  flex: 1;
  padding: 7px 10px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 14px;
}
.role-name-input:disabled { opacity: 0.5; }

.role-color-input {
  width: 36px;
  height: 36px;
  border-radius: 6px;
  border: 1px solid var(--border);
  padding: 2px;
  cursor: pointer;
  background: transparent;
}

.perm-section {
  margin-bottom: 16px;
}

.perm-label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.perm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 0;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  user-select: none;
}
.perm-row.perm-disabled { opacity: 0.5; }
.perm-row input[type=checkbox] { accent-color: var(--accent); }

.role-editor-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
}

.role-assign-section {
  border-top: 1px solid var(--border);
  padding-top: 14px;
}

.role-members {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
}

.role-member-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 6px;
  border-radius: 5px;
  font-size: 13px;
  color: var(--text-primary);
  background: var(--bg-hover);
}

.role-assign-add {
  display: flex;
  gap: 8px;
  align-items: center;
}

.assign-select {
  flex: 1;
  padding: 6px 8px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
}

/* Инвайты */
.invite-create {
  background: var(--bg-hover);
  border-radius: 8px;
  padding: 14px;
  margin-bottom: 14px;
}

.invite-create-fields {
  display: flex;
  gap: 12px;
  margin-bottom: 10px;
}

.invite-field {
  flex: 1;
}

.invite-field label {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.invite-input {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
}

.invite-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.invite-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  background: var(--bg-hover);
  border-radius: 7px;
}

.invite-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.invite-code {
  font-family: monospace;
  font-size: 14px;
  font-weight: 700;
  color: var(--accent);
}

.invite-meta {
  font-size: 12px;
  color: var(--text-muted);
}

.invite-actions {
  display: flex;
  gap: 4px;
}

/* Утилиты кнопок */
.btn-primary {
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 7px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-primary:hover { opacity: 0.9; }

.btn-secondary {
  background: var(--bg-active);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 7px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.btn-secondary:hover { background: var(--bg-hover); }
.btn-secondary:disabled { opacity: 0.4; cursor: not-allowed; }

.btn-danger {
  background: rgba(230, 57, 70, 0.12);
  color: #e63946;
  border: 1px solid rgba(230, 57, 70, 0.3);
  border-radius: 6px;
  padding: 7px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.btn-danger:hover { background: rgba(230, 57, 70, 0.2); }

.btn-sm {
  padding: 5px 10px;
  font-size: 12px;
}

.empty-state {
  color: var(--text-muted);
  font-size: 13px;
  text-align: center;
  padding: 20px 0;
}
</style>

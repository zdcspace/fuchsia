// Copyright 2016 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef APPS_MODULAR_SRC_STORY_RUNNER_STORY_PROVIDER_IMPL_H_
#define APPS_MODULAR_SRC_STORY_RUNNER_STORY_PROVIDER_IMPL_H_

#include <memory>
#include <unordered_map>
#include <unordered_set>

#include "application/services/application_environment.fidl.h"
#include "apps/ledger/services/public/ledger.fidl.h"
#include "apps/modular/lib/fidl/operation.h"
#include "apps/modular/services/story/story_data.fidl.h"
#include "apps/modular/services/story/story_provider.fidl.h"
#include "apps/modular/src/agent_runner/agent_runner.h"
#include "apps/modular/src/component/component_context_impl.h"
#include "apps/modular/src/component/message_queue_manager.h"
#include "apps/modular/src/story_runner/conflict_resolver_impl.h"
#include "apps/modular/src/story_runner/story_storage_impl.h"
#include "lib/fidl/cpp/bindings/binding_set.h"
#include "lib/fidl/cpp/bindings/interface_ptr.h"
#include "lib/fidl/cpp/bindings/interface_ptr_set.h"
#include "lib/fidl/cpp/bindings/interface_request.h"
#include "lib/fidl/cpp/bindings/string.h"
#include "lib/ftl/macros.h"

namespace modular {
class Resolver;
class StoryImpl;

namespace {
class DeleteStoryCall;
}  // namespace

class StoryProviderImpl : public StoryProvider, ledger::PageWatcher {
 public:
  StoryProviderImpl(app::ApplicationEnvironmentPtr environment,
                    fidl::InterfaceHandle<ledger::Ledger> ledger,
                    const std::string& device_name,
                    const ComponentContextInfo& component_context_info);

  ~StoryProviderImpl() override;

  void AddBinding(fidl::InterfaceRequest<StoryProvider> request);

  // Called by empty binding set handler of StoryImpl to remove the
  // corresponding entry.
  void PurgeController(const std::string& story_id);

  // Obtains the StoryData for an existing story from the ledger.
  void GetStoryData(const fidl::String& story_id,
                    const std::function<void(StoryDataPtr)>& result);

  // Used by CreateStory() to write story meta-data to the ledger.
  void WriteStoryData(StoryDataPtr story_data, std::function<void()> done);

  app::ApplicationLauncher* launcher() { return launcher_.get(); }

  const ComponentContextInfo& component_context_info() {
    return component_context_info_;
  }

  // Used by StoryImpl.
  using Storage = StoryStorageImpl::Storage;
  std::shared_ptr<Storage> storage() { return storage_; }
  ledger::PagePtr GetStoryPage(const fidl::Array<uint8_t>& story_page_id);

  using FidlStringMap = fidl::Map<fidl::String, fidl::String>;

 private:
  // |StoryProvider|
  void CreateStory(const fidl::String& url,
                   const CreateStoryCallback& callback) override;

  // |StoryProvider|
  void CreateStoryWithInfo(
      const fidl::String& url,
      FidlStringMap extra_info,
      const fidl::String& root_json,
      const CreateStoryWithInfoCallback& callback) override;

  // |StoryProvider|
  void DeleteStory(const fidl::String& story_id,
                   const DeleteStoryCallback& callback) override;

  // |StoryProvider|
  void GetStoryInfo(const fidl::String& story_id,
                    const GetStoryInfoCallback& callback) override;

  // |StoryProvider|
  void GetController(const fidl::String& story_id,
                     fidl::InterfaceRequest<StoryController> request) override;

  // |StoryProvider|
  void PreviousStories(const PreviousStoriesCallback& callback) override;

  // |StoryProvider|
  void Watch(fidl::InterfaceHandle<StoryProviderWatcher> watcher) override;

  // |PageWatcher|
  void OnChange(ledger::PageChangePtr page,
                const OnChangeCallback& callback) override;

  // Every time we receive an OnChange notification, we update the
  // root page snapshot so we see the current state. Just in case, we
  // also install a connection error handler on the snapshot
  // connection, so we can log when the connection unexepctedly closes
  // (although we cannot do anything else about it).
  fidl::InterfaceRequest<ledger::PageSnapshot> ResetRootSnapshot();

  app::ApplicationEnvironmentPtr environment_;
  ledger::LedgerPtr ledger_;
  ConflictResolverImpl conflict_resolver_;

  fidl::BindingSet<StoryProvider> bindings_;

  // We can only accept binding requests once the instance is fully
  // initalized. So we queue them up initially.
  bool ready_{};
  std::vector<fidl::InterfaceRequest<StoryProvider>> requests_;

  app::ApplicationLauncherPtr launcher_;

  // A list of IDs of *all* stories available on a user's ledger.
  std::unordered_set<std::string> story_ids_;

  // This is a container of all operations that are currently enqueued
  // to run in a FIFO manner. All operations exposed via
  // |StoryProvider| interface are queued here.
  //
  // The advantage of doing this is that if an operation consists of
  // multiple asynchronous calls then no state needs to be maintained
  // for incomplete / pending operations.
  //
  // TODO(mesch,alhaad): At some point we might want to increase
  // concurrency and have one operation queue per story for those
  // operations that only affect one story.
  //
  // TODO(mesch): No story provider operation can invoke a story
  // controller operation that would cause the story controller
  // updating its story info state, because that would be a nested
  // operation and it would deadlock. There are no such operations
  // right now, but we need to establish a pattern that makes it
  // simply and obvious how to not introduce deadlocks.
  OperationQueue operation_queue_;

  std::shared_ptr<Storage> storage_;

  // The root page that we read from.
  ledger::PagePtr root_page_;

  // The current snapshot of the root page we read from, as obtained
  // from the watcher on the root page. It is held by a shared
  // pointer, because we may update it while Operation instances that
  // read from it are still in progress, and they need to hold on to
  // the same snapshot they started with, lest the methods called on
  // that snapshot never return. PageSnaphotPtr cannot be duplicated,
  // because it doesn't have a duplicate method.
  std::shared_ptr<ledger::PageSnapshotPtr> root_snapshot_;

  fidl::Binding<ledger::PageWatcher> page_watcher_binding_;

  fidl::InterfacePtrSet<StoryProviderWatcher> watchers_;

  std::unordered_map<std::string, std::unique_ptr<StoryImpl>>
      story_controllers_;

  // This represents a delete operation that was created via |DeleteStory()|
  // but is awaiting the corresponding |PageWatcher::OnChange()| before the
  // operation can be marked as done.
  //
  // Note that delete operations taking place on remote devices can still
  // trigger a new delete operation.
  std::pair<std::string, DeleteStoryCall*> pending_deletion_;

  ComponentContextInfo component_context_info_;

  FTL_DISALLOW_COPY_AND_ASSIGN(StoryProviderImpl);
};

}  // namespace modular

#endif  // APPS_MODULAR_SRC_STORY_RUNNER_STORY_PROVIDER_IMPL_H_
